use std::str::FromStr;
use std::sync::Arc;

mod alias;
mod error;
mod sort;

mod get;
mod delete;
mod index;
mod patch;
mod post;
mod remove;
mod replace;

pub mod async;
pub mod raw;
pub mod rel;

pub use self::alias::{GetAliased, IndexAliased, AliasRequest};
pub use self::error::Error;

pub use self::get::Get;
pub use self::delete::Delete;
pub use self::index::Index;
pub use self::patch::Patch;
pub use self::post::Post;
pub use self::remove::Remove;
pub use self::replace::Replace;

pub trait Resource: Sized + 'static {
    type Id: ToString + FromStr + PartialEq + Clone;
    fn id(&self) -> Self::Id;
    fn resource() -> &'static str;
    fn resource_plural() -> &'static str;
}

pub enum Entity<T: Resource> {
    Id(T::Id),
    Resource(Arc<T>),
}

impl<T: Resource> Clone for Entity<T> {
    fn clone(&self) -> Entity<T> {
        match *self {
            Entity::Id(ref id)              => Entity::Id(id.clone()),
            Entity::Resource(ref resource)  => Entity::Resource(resource.clone()),
        }
    }
}

impl Resource for () {
    type Id = String;
    fn id(&self) -> Self::Id { String::new() }
    fn resource() -> &'static str { "" }
    fn resource_plural() -> &'static str { "" }
}
