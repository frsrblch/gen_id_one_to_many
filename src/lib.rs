use force_derive::{ForceClone, ForceDefault};
use gen_id_allocator::{Id, ValidId};
use gen_id_component::{Component, RawComponent};
use ref_cast::RefCast;
use std::collections::HashSet;

#[derive(Debug, ForceDefault, ForceClone)]
pub struct OneToMany<Source, Target> {
    targets: RawComponent<Source, HashSet<Id<Target>>>,
    source: RawComponent<Target, Option<Id<Source>>>,
}

impl<Source, Target> OneToMany<Source, Target> {
    #[inline]
    pub fn source(&self) -> &Component<Target, Option<Id<Source>>> {
        Component::ref_cast(&self.source)
    }

    #[inline]
    pub fn targets(&self) -> &Component<Source, HashSet<Id<Target>>> {
        Component::ref_cast(&self.targets)
    }

    #[inline]
    pub fn link<S, T>(&mut self, source: S, target: T)
    where
        S: ValidId<Arena = Source>,
        T: ValidId<Arena = Target>,
    {
        self.link_inner(source.id(), target.id());
    }

    #[inline]
    fn link_inner(&mut self, source: Id<Source>, target: Id<Target>) {
        self.unlink_inner(target);

        self.source.insert_with(target, Some(source), || None);
        if let Some(targets) = self.targets.get_mut(source) {
            targets.insert(target);
        } else {
            let mut set = HashSet::with_capacity(4);
            set.insert(target);
            self.targets.insert_with(source, set, || HashSet::new());
        }
    }

    #[inline]
    pub fn unlink<T>(&mut self, target: T)
    where
        T: ValidId<Arena = Target>,
    {
        self.unlink_inner(target.id());
    }

    #[inline]
    fn unlink_inner(&mut self, target: Id<Target>) {
        if let Some(existing_source) = self.source.remove(target) {
            if let Some(targets) = self.targets.get_mut(existing_source) {
                targets.remove(&target);
            }
        }
    }

    #[inline]
    pub fn unlink_source<S: ValidId<Arena = Source>>(&mut self, source: S) {
        self.unlink_source_inner(source.id());
    }

    #[inline]
    fn unlink_source_inner(&mut self, source: Id<Source>) {
        if let Some(targets) = self.targets.get_mut(source) {
            for target in targets.drain() {
                self.source.remove(target);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use gen_id_allocator::fixed_id;

    #[derive(Debug)]
    struct Source;
    fixed_id!(Source);

    #[derive(Debug)]
    struct Target;
    fixed_id!(Target);

    macro_rules! set {
        ( $($value:expr $(,)?)* ) => {
            vec![$($value,)*].into_iter().collect::<std::collections::HashSet<_>>()
        };
    }

    #[test]
    fn link() {
        let mut links = OneToMany::<Source, Target>::default();
        let s0 = Id::first(0);
        let t0 = Id::first(0);
        let t1 = Id::first(1);

        links.link(s0, t0);
        links.link(s0, t1);

        assert_eq!(&set![t0, t1], &links.targets[s0]);
        assert_eq!(Some(s0), links.source[t0]);
        assert_eq!(Some(s0), links.source[t1]);
    }

    #[test]
    fn relink() {
        let mut links = OneToMany::<Source, Target>::default();
        let s0 = Id::first(0);
        let s1 = Id::first(1);
        let t0 = Id::first(0);
        let t1 = Id::first(1);

        links.link(s0, t0);
        links.link(s0, t1);

        links.link(s1, t0);

        assert_eq!(&set![t1], &links.targets[s0]);
        assert_eq!(&set![t0], &links.targets[s1]);
        assert_eq!(Some(s1), links.source[t0]);
        assert_eq!(Some(s0), links.source[t1]);
    }

    #[test]
    fn unlink() {
        let mut links = OneToMany::<Source, Target>::default();
        let s0 = Id::first(0);
        let t0 = Id::first(0);

        links.link(s0, t0);
        links.unlink(t0);

        assert!(links.targets()[s0].is_empty());
        assert_eq!(None, links.source[t0]);
    }

    #[test]
    fn unlink_source() {
        let mut links = OneToMany::<Source, Target>::default();
        let s0 = Id::first(0);
        let t0 = Id::first(0);
        let t1 = Id::first(1);

        links.link(s0, t0);
        links.link(s0, t1);
        links.unlink_source(s0);

        assert!(links.targets[s0].is_empty());
        assert_eq!(None, links.source[t0]);
        assert_eq!(None, links.source[t1]);
    }

    #[test]
    fn sparse_insert() {
        let mut links = OneToMany::<Source, Target>::default();
        let s0 = Id::first(0);
        let s1 = Id::first(1);
        let t0 = Id::first(0);

        links.link(s1, t0);

        assert!(links.targets[s0].is_empty());
        assert_eq!(&set![t0], &links.targets[s1]);
        assert_eq!(Some(s1), links.source[t0]);
    }

    #[test]
    fn unlink_empty_target_doesnt_panic() {
        let mut links = OneToMany::<Source, Target>::default();
        let t0 = Id::first(1);

        links.unlink(t0);
    }

    #[test]
    fn unlink_empty_source_doesnt_panic() {
        let mut links = OneToMany::<Source, Target>::default();
        let s0 = Id::first(1);

        links.unlink_source(s0);
    }
}
