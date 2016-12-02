#![allow(unused_parens)]

use api::{self, Error};
use api::rel;
use api::raw;
use router::{Request, Router, ResourceRoute, Method};
use futures::{IntoFuture, Future};
use receiver::{Receiver};
use presenter::Presenter;

macro_rules! try_status {
    ($x:expr, $p:expr)  => {
        match $x {
            Ok(x)   => x,
            Err(_)  => {
                return Box::new(Ok($p.present_err(Error::Conflict)).into_future())
            }
        }
    };
}

pub struct _Router<'a, R: Router + 'a> {
    pub router: &'a mut R,
}

impl<'a, R: Router> _Router<'a, R> {
    pub fn new(router: &'a mut R) -> _Router<'a, R> {
        _Router {
            router: router,
        }
    }

    pub fn attach_replace_many<T, Rel, P, C>(&mut self)
    where
        T: rel::raw::ReplaceMany<<P as Presenter<Rel::Resource, R>>::Include, Rel>,
        Rel: rel::ToMany,
        Rel::Resource: raw::RawResource,
        P: Presenter<Rel::Resource, R> + Presenter<(), R>,
        C: Receiver<Rel::Resource, R::Request>,
    {
        fn replace_many<R, T, Rel, P, C>(request: R::Request, link_maker: R::LinkMaker) -> Box<Future<Item = R::Response, Error = ()>>
        where
            R: Router,
            T: rel::raw::ReplaceMany<P::Include, Rel>,
            Rel: rel::ToMany,
            Rel::Resource: raw::RawResource,
            P: Presenter<Rel::Resource, R>,
            C: Receiver<Rel::Resource, R::Request>,
        {
            let options = request.collection_options();
            let presenter = P::prepare(options.field_set, link_maker);
            let id = match request.id() {
                Some(id)    => try_status!(id.parse(), presenter),
                None        => try_status!(Err(()), presenter),
            };
            let received = try_status!(C::receive_collection(request), presenter);
            presenter.try_present(T::replace_many(api::Entity::Id(id), received))
        }
        fn replace_many_rel<R, T, Rel, P, C>(request: R::Request, link_maker: R::LinkMaker) -> Box<Future<Item = R::Response, Error = ()>>
        where
            R: Router,
            T: rel::raw::ReplaceMany<<P as Presenter<Rel::Resource, R>>::Include, Rel>,
            Rel: rel::ToMany,
            Rel::Resource: raw::RawResource,
            P: Presenter<Rel::Resource, R> + Presenter<(), R>,
            C: Receiver<Rel::Resource, R::Request>,
        {
            let options = request.resource_options();
            let presenter = <P as Presenter<(), R>>::prepare(options.field_set, link_maker);
            let id = match request.id() {
                Some(id)    => match id.parse() {
                    Ok(id)  => id,
                    Err(_)  => return Box::new(Ok(<P as Presenter<(), R>>::present_err(presenter, Error::Conflict)).into_future()),
                },
                None => return Box::new(Ok(<P as Presenter<(), R>>::present_err(presenter, Error::Conflict)).into_future()),
            };
            let identifiers = match C::receive_to_many::<Rel>(request) {
                Ok(rel) => rel,
                Err(_)  => return Box::new(Ok(<P as Presenter<(), R>>::present_err(presenter, Error::Conflict)).into_future()),
            };
            let rel_ids: Vec<_> = {
                let mut ids = vec![];
                for identifier in identifiers {
                    match identifier.id.parse() {
                        Ok(id)  => ids.push(id),
                        Err(_)  => return Box::new(Ok(<P as Presenter<(), R>>::present_err(presenter, Error::Conflict)).into_future()),
                    }
                }
                ids
            };
            presenter.try_present(T::replace_links(api::Entity::Id(id), &rel_ids))
        }
        self.router.attach_resource(T::resource_plural(), ResourceRoute {
            method: Method::Replace,
            relation: Some((Rel::to_many(), false))
        }, replace_many::<R, T, Rel, P, C>);
        self.router.attach_resource(T::resource_plural(), ResourceRoute {
            method: Method::Replace,
            relation: Some((Rel::to_many(), true))
        }, replace_many_rel::<R, T, Rel, P, C>);
    }

    pub fn attach_get_alias<T, P>(&mut self, route: &'static str)
    where
        T: raw::RawGetAliased<P::Include>,
        P: Presenter<T, R>,
    { 
        fn get_aliased<R, T, P>(request: R::Request, link_maker: R::LinkMaker) -> Box<Future<Item = R::Response, Error = ()>>
        where
            T: raw::RawGetAliased<P::Include>,
            P: Presenter<T, R>,
            R: Router,
        {
            let options = request.resource_options();
            let presenter = P::prepare(options.field_set, link_maker);
            presenter.try_present(T::get(request.alias_info(), options.includes))
        }
        self.router.attach_alias(route, Method::Get, get_aliased::<R, T, P>);
    }

    pub fn attach_index_aliased<T, P>(&mut self, route: &'static str)
    where
        T: raw::RawIndexAliased<P::Include>,
        P: Presenter<T, R>,
    {
        fn index_aliased<R, T, P>(request: R::Request, link_maker: R::LinkMaker) -> Box<Future<Item = R::Response, Error = ()>>
        where
            T: raw::RawIndexAliased<P::Include>,
            P: Presenter<T, R>,
            R: Router,
        {
            let options = request.collection_options();
            let presenter = P::prepare(options.field_set, link_maker);
            presenter.try_present(T::index(request.alias_info(), options.includes, options.sort))
        }
        self.router.attach_alias(route, Method::Index, index_aliased::<R, T, P>);
    }
}
