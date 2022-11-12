//! # DBus interface proxies for: `org.a11y.atspi.Event.Object`, `org.a11y.atspi.Event.Window`, `org.a11y.atspi.Event.Mouse`, `org.a11y.atspi.Event.Keyboard`, `org.a11y.atspi.Event.Terminal`, `org.a11y.atspi.Event.Document`, `org.a11y.atspi.Event.Focus`
//!
//! This code was generated by `zbus-xmlgen` `2.0.1` from DBus introspection data.
//! Source: `Event.xml`.
//!
//! You may prefer to adapt it, instead of using it verbatim.
//!
//! More information can be found in the
//! [Writing a client proxy](https://dbus.pages.freedesktop.org/zbus/client.html)
//! section of the zbus documentation.
//!

pub mod document;
pub mod focus;
pub mod keyboard;
pub mod mouse;
pub mod object;
pub mod terminal;
pub mod window;

use std::{collections::HashMap, sync::Arc};

use serde::{Deserialize, Serialize};
use zbus::{
    names::{InterfaceName, MemberName, UniqueName},
    zvariant::{self, OwnedObjectPath, OwnedValue, Signature, Type, Value},
    Message,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct EventBody<'a, T> {
    #[serde(rename = "type")]
    pub kind: T,
    pub detail1: i32,
    pub detail2: i32,
    #[serde(borrow)]
    pub any_data: Value<'a>,
    #[serde(borrow)]
    pub properties: HashMap<&'a str, Value<'a>>,
}

impl<T> Type for EventBody<'_, T> {
    fn signature() -> Signature<'static> {
        <(&str, i32, i32, Value, HashMap<&str, Value>)>::signature()
    }
}

#[derive(Debug, Serialize, Deserialize, Type)]
pub struct EventBodyQT {
    #[serde(rename = "type")]
    pub kind: String,
    pub detail1: i32,
    pub detail2: i32,
    pub any_data: OwnedValue,
    pub properties: (String, OwnedObjectPath),
}

#[derive(Clone, Debug, Serialize, Deserialize, Type)]
pub struct EventBodyOwned {
    #[serde(rename = "type")]
    pub kind: String,
    pub detail1: i32,
    pub detail2: i32,
    pub any_data: OwnedValue,
    pub properties: HashMap<String, OwnedValue>,
}

impl From<EventBodyQT> for EventBodyOwned {
    fn from(body: EventBodyQT) -> Self {
        let mut props = HashMap::new();
        props.insert(body.properties.0, Value::ObjectPath(body.properties.1.into_inner()).to_owned());
        Self {
            kind: body.kind,
            detail1: body.detail1,
            detail2: body.detail2,
            any_data: body.any_data,
            properties: props,
        }
    }
}

pub struct Event {
    message: Arc<Message>,
    body: EventBodyOwned,
}

impl TryFrom<Arc<Message>> for Event {
    type Error = zbus::Error;

    fn try_from(message: Arc<Message>) -> zbus::Result<Self> {
        let qt_sig = Signature::try_from("siiv(so)").unwrap();
        let body: EventBodyOwned = match message.body_signature() {
            Ok(sig) => {
                if sig == qt_sig {
                    EventBodyOwned::from(message.body::<EventBodyQT>()?)
                } else {
                    message.body::<EventBodyOwned>()?
                }
            }
            Err(e) => return Err(e),
        };
        Ok(Self { message, body })
    }
}

impl Event {
    pub fn sender(&self) -> zbus::Result<Option<UniqueName>> {
        Ok(self.message.header()?.sender()?.cloned())
    }

    pub fn path(&self) -> Option<zvariant::ObjectPath> {
        self.message.path()
    }

    /// Returns the atspi event string for this event type (E.G. "Object:StateChanged:Focused").
    ///
    /// This should not be used for matching on events as it needlessly allocates and copies the 3
    /// components of the event type. It is meant for logging, etc.
    pub fn event_string(&self) -> String {
        let interface = self.interface().expect("Event should have an interface");
        let interface = interface
            .rsplit('.')
            .next()
            .expect("Interface should contain a '.'");
        let member = self.member().expect("Event should have a member");
        let kind = self.kind();
        format!("{interface}:{member}:{kind}")
    }

    /// For now this returns the full interface name because the lifetimes in [`zbus_names`] are
    /// wrong such that the `&str` you can get from a
    /// [`zbus_names::InterfaceName`][zbus::names::InterfaceName] is tied to the lifetime of that
    /// name, not to the lifetime of the message as it should be. In future, this will return only
    /// the last component of the interface name (I.E. "Object" from
    /// "org.a11y.atspi.Event.Object").
    pub fn interface(&self) -> Option<InterfaceName<'_>> {
        self.message.interface()
    }

    pub fn member(&self) -> Option<MemberName<'_>> {
        self.message.member()
    }

    pub fn kind(&self) -> &str {
        &self.body.kind
    }

    pub fn detail1(&self) -> i32 {
        self.body.detail1
    }

    pub fn detail2(&self) -> i32 {
        self.body.detail2
    }

    pub fn any_data(&self) -> &zvariant::OwnedValue {
        &self.body.any_data
    }

    pub fn properties(&self) -> &HashMap<String, zvariant::OwnedValue> {
        &self.body.properties
    }

    pub fn message(&self) -> &Arc<Message> {
        &self.message
    }

    pub fn body(&self) -> &EventBodyOwned {
        &self.body
    }
}
