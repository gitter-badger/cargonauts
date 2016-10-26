#[macro_use]
mod relation;

/// The entry point for the routes DSL, which defines the endpoints of your API.
#[macro_export]
macro_rules! routes {
    {$(resource $resource:ident : [$($method:ident),*] { $(has $count:ident $rel:ident : [$($rel_method:ident),*];)*})*} => {
        pub fn attach_routes<T: $crate::router::Router>(router: &mut T) {
            let mut router = $crate::_internal::_Router::new(router);
            $({ _resource!(router, $resource : [$($method),*] {$($count $rel: [$($rel_method),*];)*}); })*
        }
    }
}

/// Do not call this macro, it is an implementation detail of the routes! macro.
#[macro_export]
macro_rules! _resource {
    ($router:expr, $resource:ty: [] $ignore:tt) => { };
    ($router:expr, $resource:ty: [$($method:ident),*] {}) => {
        impl $crate::api::raw::RawFetch for $resource {
            type Relationships = ();
        }

        impl $crate::api::raw::RawUpdate for $resource {
            type Relationships = ();
        }

        impl $crate::_internal::_FetchRels for $resource {
            fn rels(_: &$crate::api::Entity<Self>, _: &[String]) -> Result<(Self::Relationships, Vec<$crate::api::raw::Include>), $crate::api::Error> {
                Ok(((), vec![]))
            }
        }

        impl $crate::_internal::_UpdateRels for $resource {
            fn update_rels(_: &$crate::api::Entity<Self>, rels: ()) -> Result<(), $crate::api::Error> {
                Ok(())
            }
        }
        _methods!($router, $resource, [$($method),*]);

    };
    ($router:expr, $resource:ty: [$($method:ident),*] { $($count:ident $rel:ident : [$($rel_method:ident),*];)* }) => {
        impl $crate::api::raw::RawFetch for $resource {
            type Relationships = Relationships;
        }

        impl $crate::api::raw::RawUpdate for $resource {
            type Relationships = UpdateRelationships;
        }

        impl $crate::_internal::_FetchRels for $resource {
            fn rels(id: &$crate::api::Entity<Self>, includes: &[String]) -> Result<(Self::Relationships, Vec<$crate::api::raw::Include>), $crate::api::Error> {
                let mut include_objects = vec![];
                let rels = Relationships {
                    $(
                        $rel: {
                            _fetch_rel!(id, includes, include_objects, $resource, $rel, $count)
                        },
                    )*
                };
                Ok((rels, include_objects))
            }
        }

        impl $crate::_internal::_UpdateRels for $resource {
            fn update_rels(id: &$crate::api::Entity<Self>, rels: UpdateRelationships) -> Result<Relationships, $crate::api::Error> {
                let (mut current_rels, _) = try!(<$resource as $crate::_internal::_FetchRels>::rels(id, &[]));
                $(
                    if let Some(rel) = rels.$rel {
                        _link_rel!(id, rel, $resource, $rel, $count);
                        current_rels.$rel = rel;
                    };
                )*
                Ok(current_rels)
            }
        }

        #[allow(non_snake_case)]
        struct Relationships {
            $(
                $rel: $crate::api::raw::Relationship,
            )*
        }

        impl<'a> $crate::api::raw::FetchRelationships<'a> for Relationships {
            type Iter = RelationshipsIter<'a>;

            fn iter(&'a self) -> RelationshipsIter<'a> {
                RelationshipsIter {
                    rels: self,
                    state: RelationshipsEnum::Init,
                }
            }

            fn count(&self) -> usize {
                _count_rels!($($rel),*)
            }
        }

        struct RelationshipsIter<'a> {
            rels: &'a Relationships,
            state: RelationshipsEnum,
        }

        impl<'a> Iterator for RelationshipsIter<'a> {
            type Item = (&'a str, &'a $crate::api::raw::Relationship);
            fn next(&mut self) -> Option<Self::Item> {
                match self.state {
                    RelationshipsEnum::Init => $( {
                        self.state = RelationshipsEnum::$rel;
                        Some((_name_rel!($rel, $count), &self.rels.$rel))
                    }
                    RelationshipsEnum::$rel => )* { None }
                }
            }
        }

        #[derive(Copy, Clone, Eq, PartialEq, Debug)]
        enum RelationshipsEnum {
            Init,
            $($rel,)*
        }

        #[derive(Clone, Eq, PartialEq, Debug, Default)]
        #[allow(non_snake_case)]
        struct UpdateRelationships {
            $(
                $rel: Option<$crate::api::raw::Relationship>,
            )*
        }

        impl $crate::api::raw::UpdateRelationships for UpdateRelationships {
            fn from_iter<I>(iter: I) -> Result<Self, $crate::api::Error> where I: Iterator<Item = (String, $crate::api::raw::Relationship)> {
                let mut rels = UpdateRelationships::default();
                for (name, value) in iter {
                    $( if &name[..] == _name_rel!($rel, $count) {
                        rels.$rel = Some(value);
                    } else )* {
                        return Err($crate::api::Error::BadRequest)
                    }
                }
                Ok(rels)
            }
        }

        _methods!($router, $resource, [$($method),*]);
        $(_rel_methods!($router, $resource, $count $rel [$($rel_method),*]);)*
    };
}

/// Do not call this macro, it is an implementation detail of the routes! macro.
#[macro_export]
macro_rules! _methods {
    ($router:expr, $resource:ty, [delete, $($method:ident),*]) => {
        $router.attach_delete::<$resource>();
        _methods!($router, $resource, [$($method),*])
    };
    ($router:expr, $resource:ty, [delete]) => {
        $router.attach_delete::<$resource>()
    };
    ($router:expr, $resource:ty, [get, $($method:ident),*]) => {
        $router.attach_get::<$resource>();
        _methods!($router, $resource, [$($method),*]);
    };
    ($router:expr, $resource:ty, [get]) => {
        $router.attach_get::<$resource>();
    };
    ($router:expr, $resource:ty, [index, $($method:ident),*]) => {
        $router.attach_index::<$resource>();
        _methods!($router, $resource, [$($method),*])
    };
    ($router:expr, $resource:ty, [index]) => {
        $router.attach_index::<$resource>();
    };
    ($router:expr, $resource:ty,  [patch, $($method:ident),*]) => {
        $router.attach_patch::<$resource>();
        _methods!($router, $resource, [$($method),*]);
    };
    ($router:expr, $resource:ty, [patch]) => {
        $router.attach_patch::<$resource>();
    };
    ($router:expr, $resource:ty, [post, $($method:ident),*]) => {
        $router.attach_post::<$resource>();
        _methods!($router, $resource, [$($method),*])
    };
    ($router:expr, $resource:ty, [post]) => {
        $router.attach_post::<$resource>();
    };
    ($router:expr, $resource:ty, []) => {
    };
}

#[macro_export]
macro_rules! _rel_methods {
    ($router:expr, $resource:ty, one $rel:ident [fetch, $($method:ident),*])  => {
        $router.attach_fetch_one::<$resource, $rel>();
        _rel_methods!($router, $resource, one $rel [$($method),*]);
    };
    ($router:expr, $resource:ty, one $rel:ident [fetch])  => {
        $router.attach_fetch_one::<$resource, $rel>();
    };
    ($router:expr, $resource:ty, one $rel:ident [post, $($method:ident),*])  => {
        $router.attach_post_one::<$resource, $rel>();
        _rel_methods!($router, $resource, one $rel [$($method),*]);
    };
    ($router:expr, $resource:ty, one $rel:ident [post])  => {
        $router.attach_post_one::<$resource, $rel>();
    };
    ($router:expr, $resource:ty, one $rel:ident [patch, $($method:ident),*])  => {
        $router.attach_patch_one::<$resource, $rel>();
        _rel_methods!($router, $resource, one $rel [$($method),*]);
    };
    ($router:expr, $resource:ty, one $rel:ident [patch])  => {
        $router.attach_patch_one::<$resource, $rel>();
    };
    ($router:expr, $resource:ty, one $rel:ident [delete, $($method:ident),*])  => {
        $router.attach_delete_one::<$resource, $rel>();
        _rel_methods!($router, $resource, one $rel [$($method),*]);
    };
    ($router:expr, $resource:ty, one $rel:ident [delete])  => {
        $router.attach_delete_one::<$resource, $rel>();
    };
    ($router:expr, $resource:ty, many $rel:ident [fetch, $($method:ident),*])  => {
        $router.attach_fetch_many::<$resource, $rel>();
        _rel_methods!($router, $resource, many $rel [$($method),*]);
    };
    ($router:expr, $resource:ty, many $rel:ident [fetch])  => {
        $router.attach_fetch_many::<$resource, $rel>();
    };
    ($router:expr, $resource:ty, many $rel:ident [append, $($method:ident),*])  => {
        $router.attach_append_many::<$resource, $rel>();
        _rel_methods!($router, $resource, many $rel [$($method),*]);
    };
    ($router:expr, $resource:ty, many $rel:ident [append])  => {
        $router.attach_append_many::<$resource, $rel>();
    };
    ($router:expr, $resource:ty, many $rel:ident [replace, $($method:ident),*])  => {
        $router.attach_replace_many::<$resource, $rel>();
        _rel_methods!($router, $resource, many $rel [$($method),*]);
    };
    ($router:expr, $resource:ty, many $rel:ident [replace])  => {
        $router.attach_replace_many::<$resource, $rel>();
    };
    ($router:expr, $resource:ty, many $rel:ident [remove, $($method:ident),*])  => {
        $router.attach_remove_many::<$resource, $rel>();
        _rel_methods!($router, $resource, many $rel [$($method),*]);
    };
    ($router:expr, $resource:ty, many $rel:ident [remove])  => {
        $router.attach_remove_many::<$resource, $rel>();
    };
    ($router:expr, $resource:ty, many $rel:ident [clear, $($method:ident),*])  => {
        $router.attach_clear_many::<$resource, $rel>();
        _rel_methods!($router, $resource, many $rel [$($method),*]);
    };
    ($router:expr, $resource:ty, many $rel:ident [clear])  => {
        $router.attach_clear_many::<$resource, $rel>();
    };
    ($router:expr, $resource:ty, $count:ident $rel:ident [])  => {};
}

#[macro_export]
macro_rules! _fetch_rel {
    ($id:expr, $includes:expr, $includes_out:expr, $resource:ty, $rel:ty, one) => {
        if $includes.iter().any(|include| include == _name_rel!($rel, one)) {
            match try!(<$resource as $crate::api::rel::raw::FetchOne<$rel>>::fetch_one($id, &[])) {
                Some(response)  => {
                    let identifier = $crate::api::raw::Identifier::from(&response.resource);
                    $includes_out.push(response.resource.into());
                    $crate::api::raw::Relationship::One(Some(identifier))
                }
                None            => {
                    $crate::api::raw::Relationship::One(None)
                }
            }
        } else {
            let identifier = match try!(<$resource as $crate::api::rel::HasOne<$rel>>::has_one($id)) {
                Some(id)    => {
                    Some($crate::api::raw::Identifier::new::<<$rel as $crate::api::rel::Relation>::Resource>(&id))
                }
                None        => None,
            };
            $crate::api::raw::Relationship::One(identifier)
        };
    };
    ($id:expr, $includes:expr, $includes_out:expr, $resource:ty, $rel:ty, many) => {
        if $includes.iter().any(|include| include == _name_rel!($rel, many)) {
            let response = try!(<$resource as $crate::api::rel::raw::FetchMany<$rel>>::fetch_many($id, &[]));
            let identifiers = response.resources.iter().map($crate::api::raw::Identifier::from).collect();
            $includes_out.extend(response.resources.into_iter().map(Into::into));
            $crate::api::raw::Relationship::Many(identifiers)
        } else {
            let ids = try!(<$resource as $crate::api::rel::HasMany<$rel>>::has_many($id));
            let identifiers = ids.iter().map($crate::api::raw::Identifier::new::<<$rel as $crate::api::rel::Relation>::Resource>).collect();
            $crate::api::raw::Relationship::Many(identifiers)
        }
    };
}

#[macro_export]
macro_rules! _link_rel {
    ($id:expr, $rel_obj:expr, $resource:ty, $rel:ty, one)   => {
        match $rel_obj {
            $crate::api::raw::Relationship::One(Some(ref identifier)) => {
                let rel_id = try!(identifier.id.parse().or(Err($crate::api::Error::BadRequest)));
                try!(<$resource as $crate::_internal::_MaybeLinkOne<$rel>>::link_one($id, &rel_id));

            }
            $crate::api::raw::Relationship::One(None)           => {
                try!(<$resource as $crate::_internal::_MaybeUnlinkOne<$rel>>::unlink_one($id));
            }
            $crate::api::raw::Relationship::Many(_)             => {
                return Err($crate::api::Error::BadRequest);
            }
        }
    };
    ($id:expr, $rel_obj:expr, $resource:ty, $rel:ty, many)   => {
        match $rel_obj {
            $crate::api::raw::Relationship::One(_)                  => {
                return Err($crate::api::Error::BadRequest);
            }
            $crate::api::raw::Relationship::Many(ref identifiers)   => {
                let rel_ids = try!(identifiers.iter().map(|identifier| identifier.id.parse().or(Err($crate::api::Error::BadRequest))).collect::<Result<Vec<_>, _>>());
                try!(<$resource as $crate::_internal::_MaybeReplaceLinks<$rel>>::replace_links($id, &rel_ids));
            }
        }
    };
}

#[macro_export]
macro_rules! _name_rel {
    ($rel:ty, one) => {
        <$rel as $crate::api::rel::Relation>::to_one()
    };
    ($rel:ty, many) => {
        <$rel as $crate::api::rel::Relation>::to_many()
    };
}

#[macro_export]
macro_rules! _count_rels {
    ($head:ident, $($rest:ident),*) => (1 + _count_rels!($($rest),*));
    ($last:ident)                   => (1);
}