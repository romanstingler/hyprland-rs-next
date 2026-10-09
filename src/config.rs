//! # Hyprland Configuration in Rust
//!

use crate::dispatch::{DispatchType, gen_dispatch_str};
use crate::keyword::Keyword;

/// Module providing stuff for adding an removing keybinds
pub mod binds {
    use super::*;
    use crate::default_instance;
    use crate::instance::Instance;

    pub(crate) trait Join {
        fn join(&self) -> String;
    }

    /// One blanket impl instead of four near-identical ones. The old set had a
    /// copy for `Vec<Mod>`, `&[Mod]`, `Vec<Flag>` and `&[Flag]`; all four bodies
    /// were byte-identical, and every type involved is `Display`.
    impl<T, U> Join for T
    where
        T: std::ops::Deref<Target = [U]> + ?Sized,
        U: std::fmt::Display,
    {
        fn join(&self) -> String {
            let mut buf = String::new();
            for i in self.iter() {
                buf.push_str(&i.to_string());
            }
            buf
        }
    }

    /// Type for a key held by a bind
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[non_exhaustive]
    pub enum Key<'a> {
        /// Variant for if the bind holds a modded key
        Mod(
            /// Mods
            &'a [Mod],
            /// Key
            &'a str,
        ),
        /// Variant for a regular key
        Key(&'a str),
    }

    impl std::fmt::Display for Key<'_> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "{}",
                match self {
                    Key::Mod(m, s) => format!("{}_{s}", m.join()),
                    Key::Key(s) => s.to_string(),
                }
            )
        }
    }

    pub use crate::shared::Mod;

    /// Enum for bind flags
    ///
    /// The variants are the flag words upstream defines in
    /// `ConfigManager.cpp:1516-1529`; the letter a Hyprland config uses is in the
    /// `#[display]`, so renaming these changes no generated bind string.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, derive_more::Display)]
    #[non_exhaustive]
    pub enum Flag {
        /// Works when screen is locked
        #[display("l")]
        Locked,
        /// Activates on release
        #[display("r")]
        Release,
        /// Repeats when held
        #[display("e")]
        Repeat,
        /// Non-consuming, key/mouse events will be passed to the active window in addition to triggering the dispatcher.
        #[display("n")]
        NonConsuming,
        /// Used for mouse binds
        #[display("m")]
        Mouse,
        /// Transparent, cannot be shadowed by other binds.
        #[display("t")]
        Transparent,
        /// Ignore mods, will ignore modifiers.
        #[display("i")]
        IgnoreMods,
        /// Multi-key, will arbitrarily combine keys between each mod/key
        #[display("s")]
        MultiKey,
        /// Has description, will allow you to write a description for your bind.
        #[display("d")]
        HasDescription,
        /// Bypasses the app's requests to inhibit keybinds.
        #[display("p")]
        DontInhibit,
    }

    /// A struct used for indentifying bindings
    #[derive(Debug, Clone)]
    pub struct PartialBind<'a> {
        /// The modifiers used
        pub mods: &'a [Mod],
        /// The main key used
        pub key: Key<'a>,
    }

    /// A struct providing a key bind
    #[derive(Debug, Clone)]
    pub struct Binding<'a> {
        /// All the mods
        pub mods: &'a [Mod],
        /// The key
        pub key: Key<'a>,
        /// Bind flags
        pub flags: &'a [Flag],
        /// The dispatcher to be called once complete
        pub dispatcher: DispatchType<'a>,
    }

    /// Struct to hold methods for adding and removing binds
    pub struct Binder;

    impl Binder {
        pub(crate) fn gen_str_partial(PartialBind { mods, key }: PartialBind) -> String {
            format!("{},{key}", (&mods).join())
        }

        pub(crate) fn gen_str(
            Binding {
                mods,
                key,
                dispatcher,
                ..
            }: Binding,
        ) -> crate::Result<String> {
            Ok(format!(
                "{partial},{dispatcher}",
                partial = Self::gen_str_partial(PartialBind { mods, key }),
                dispatcher = gen_dispatch_str(dispatcher, false)?.data
            ))
        }

        /// Binds a keybinding
        pub fn bind(binding: Binding) -> crate::Result<()> {
            Self::instance_bind(default_instance()?, binding)
        }

        /// Unbinds a keybinding
        pub fn unbind(binding: PartialBind) -> crate::Result<()> {
            Self::instance_unbind(default_instance()?, binding)
        }

        /// Unbinds a keybinding
        pub fn instance_unbind(instance: &Instance, binding: PartialBind) -> crate::Result<()> {
            Keyword::instance_set(instance, "unbind", Self::gen_str_partial(binding))
        }

        /// Unbinds a keybinding (async)
        #[cfg(any(feature = "async-lite", feature = "tokio"))]
        pub async fn unbind_async(binding: PartialBind<'_>) -> crate::Result<()> {
            Self::instance_unbind_async(default_instance()?, binding).await
        }

        /// Unbinds a keybinding (async)
        #[cfg(any(feature = "async-lite", feature = "tokio"))]
        pub async fn instance_unbind_async(
            instance: &Instance,
            binding: PartialBind<'_>,
        ) -> crate::Result<()> {
            Keyword::instance_set_async(instance, "unbind", Self::gen_str_partial(binding)).await
        }

        /// Binds a keybinding
        pub fn instance_bind(instance: &Instance, binding: Binding) -> crate::Result<()> {
            Keyword::instance_set(
                instance,
                format!("bind{}", (&binding.flags).join()),
                Self::gen_str(binding)?,
            )
        }

        /// Binds a keybinding (async)
        #[cfg(any(feature = "async-lite", feature = "tokio"))]
        pub async fn bind_async(binding: Binding<'_>) -> crate::Result<()> {
            Self::instance_bind_async(default_instance()?, binding).await
        }

        /// Binds a keybinding (async)
        #[cfg(any(feature = "async-lite", feature = "tokio"))]
        pub async fn instance_bind_async(
            instance: &Instance,
            binding: Binding<'_>,
        ) -> crate::Result<()> {
            Keyword::instance_set_async(
                instance,
                format!("bind{}", (&binding.flags).join()),
                Self::gen_str(binding)?,
            )
            .await
        }
    }

    /// Very macro basic abstraction over [Binder] for internal use, **Dont use this instead use [crate::bind]**
    ///
    /// ```rust
    /// # use hyprland::{bind_raw, default_instance, default_instance_panic, dispatch::DispatchType, Result};
    /// #[tokio::main(flavor = "current_thread")]
    /// # async fn test() -> Result<()> {
    ///   let instance = default_instance()?;
    ///   bind_raw!(instance , &[Mod::SHIFT] , Key::Key("m")  ,  &[Flag::Locked, Flag::Release, Flag::Mouse] ,  DispatchType::Exit )?;
    ///   bind_raw!(&[Mod::SHIFT] , Key::Key("m")  ,  &[Flag::Locked, Flag::Release, Flag::Mouse] ,  DispatchType::Exit )?;
    ///   bind_raw!(async, instance, &[Mod::SHIFT] , Key::Key("m")  ,  &[Flag::Locked, Flag::Release, Flag::Mouse] ,  DispatchType::Exit).await?;
    ///   bind_raw!(async, &[Mod::SHIFT] , Key::Key("m")  ,  &[Flag::Locked, Flag::Release, Flag::Mouse] ,  DispatchType::Exit).await?;
    ///   Ok(())
    /// # }
    /// ```
    #[macro_export]
    macro_rules! bind_raw {
        (async, $instance:expr,$mods:expr,$key:expr,$flags:expr,$dis:expr ) => {{
            use $crate::config::binds::*;
            let binding = Binding {
                mods: $mods,
                key: $key,
                flags: $flags,
                dispatcher: $dis,
            };
            Binder::instance_bind_async($instance, binding)
        }};
        (async, $mods:expr,$key:expr,$flags:expr,$dis:expr ) => {{
            use $crate::config::binds::*;
            let binding = Binding {
                mods: $mods,
                key: $key,
                flags: $flags,
                dispatcher: $dis,
            };
            Binder::bind_async(binding)
        }};
        ($instance:expr,$mods:expr,$key:expr,$flags:expr,$dis:expr ) => {{
            use $crate::config::binds::*;
            let binding = Binding {
                mods: $mods,
                key: $key,
                flags: $flags,
                dispatcher: $dis,
            };
            Binder::instance_bind($instance, binding)
        }};
        ($mods:expr,$key:expr,$flags:expr,$dis:expr ) => {{
            use $crate::config::binds::*;
            let binding = Binding {
                mods: $mods,
                key: $key,
                flags: $flags,
                dispatcher: $dis,
            };
            Binder::bind(binding)
        }};
    }

    /// Macro abstraction over [Binder]
    ///
    /// ```rust
    /// # use hyprland::{bind, default_instance, dispatch::DispatchType, Result};
    /// # use hyprland::instance::Instance;
    ///
    /// #[tokio::main(flavor = "current_thread")]
    /// # async fn test() -> Result<()> {
    ///     let instance = default_instance()?;
    ///     bind!(instance, Locked Release Mouse | SHIFT, Key, "m" => Exit);
    ///     bind!(SHIFT ALT, Key, "b" => CenterWindow);
    ///     bind!(async ; Locked Release Mouse | SHIFT, Key, "m" => Exit);
    ///     bind!(async ; instance, SUPER, Key, "l" => CenterWindow);
    ///     bind!(async ; SHIFT ALT, Key, "b" => CenterWindow);
    ///     Ok(())
    /// # }
    /// ```
    #[macro_export]
    macro_rules! bind {
        (async ; $instance:expr, $( $flag:ident ) *|$( $mod:ident ) *,$keyt:ident, $( $key:expr ), * => $dis:ident, $( $arg:expr ), *) => {
            $crate::bind_raw!(
                async,
                $instance,
                &[$(Mod::$mod), *],
                Key::$keyt( $( $key ), * ),
                &[$(Flag::$flag), *],
                DispatchType::$dis( $($arg),* )
            )
        };
        (async ; $instance:expr, $( $flag:ident ) *|$( $mod:ident ) *,$keyt:ident,$( $key:expr ), * => $dis:ident ) => {
            $crate::bind_raw!(
                async,
                $instance,
                &[$(Mod::$mod), *],
                Key::$keyt( $( $key ), * ),
                &[$(Flag::$flag), *],
                DispatchType::$dis
            )
        };
        (async ; $instance:expr, $( $mod:ident ) *,$keyt:ident,$( $key:expr ), * => $dis:ident, $( $arg:expr ), *) => {
            $crate::bind_raw!(
                async,
                $instance,
                &[$(Mod::$mod), *],
                Key::$keyt( $( $key ), * ),
                &[],
                DispatchType::$dis( $($arg),* )
            )
        };
        (async ; $instance:expr, $( $mod:ident ) *,$keyt:ident,$( $key:expr ), * => $dis:ident ) => {
            $crate::bind_raw!(
                async,
                $instance,
                &[$(Mod::$mod), *],
                Key::$keyt( $( $key ), * ),
                &[],
                DispatchType::$dis
            )
        };
        (async ; $( $flag:ident ) *|$( $mod:ident ) *,$keyt:ident, $( $key:expr ), * => $dis:ident, $( $arg:expr ), *) => {
            $crate::bind_raw!(
                async,
                &[$(Mod::$mod), *],
                Key::$keyt( $( $key ), * ),
                &[$(Flag::$flag), *],
                DispatchType::$dis( $($arg),* )
            )
        };
        (async ; $( $flag:ident ) *|$( $mod:ident ) *,$keyt:ident,$( $key:expr ), * => $dis:ident ) => {
            $crate::bind_raw!(
                async,
                &[$(Mod::$mod), *],
                Key::$keyt( $( $key ), * ),
                &[$(Flag::$flag), *],
                DispatchType::$dis
            )
        };
        (async ; $( $mod:ident ) *,$keyt:ident,$( $key:expr ), * => $dis:ident, $( $arg:expr ), *) => {
            $crate::bind_raw!(
                async,
                &[$(Mod::$mod), *],
                Key::$keyt( $( $key ), * ),
                &[],
                DispatchType::$dis( $($arg),* )
            )
        };
        (async ; $( $mod:ident ) *,$keyt:ident,$( $key:expr ), * => $dis:ident ) => {
            $crate::bind_raw!(
                async,
                &[$(Mod::$mod), *],
                Key::$keyt( $( $key ), * ),
                &[],
                DispatchType::$dis
            )
        };
        ($instance:expr, $( $flag:ident ) *|$( $mod:ident ) *,$keyt:ident, $( $key:expr ), * => $dis:ident, $( $arg:expr ), *) => {
            $crate::bind_raw!(
                $instance,
                &[$(Mod::$mod), *],
                Key::$keyt( $( $key ), * ),
                &[$(Flag::$flag), *],
                DispatchType::$dis( $($arg),* )
            )
        };
        ($instance:expr, $( $flag:ident ) *|$( $mod:ident ) *,$keyt:ident,$( $key:expr ), * => $dis:ident ) => {
            $crate::bind_raw!(
                $instance,
                &[$(Mod::$mod), *],
                Key::$keyt( $( $key ), * ),
                &[$(Flag::$flag), *],
                DispatchType::$dis
            )
        };
        ($instance:expr, $( $mod:ident ) *,$keyt:ident,$( $key:expr ), * => $dis:ident, $( $arg:expr ), *) => {
            $crate::bind_raw!(
                $instance,
                &[$(Mod::$mod), *],
                Key::$keyt( $( $key ), * ),
                &[],
                DispatchType::$dis( $($arg),* )
            )
        };
        ($instance:expr, $( $mod:ident ) *,$keyt:ident,$( $key:expr ), * => $dis:ident ) => {
            $crate::bind_raw!(
                $instance,
                &[$(Mod::$mod), *],
                Key::$keyt( $( $key ), * ),
                &[],
                DispatchType::$dis
            )
        };
        ($( $flag:ident ) *|$( $mod:ident ) *,$keyt:ident, $( $key:expr ), * => $dis:ident, $( $arg:expr ), *) => {
            $crate::bind_raw!(
                &[$(Mod::$mod), *],
                Key::$keyt( $( $key ), * ),
                &[$(Flag::$flag), *],
                DispatchType::$dis( $($arg),* )
            )
        };
        ($( $flag:ident ) *|$( $mod:ident ) *,$keyt:ident,$( $key:expr ), * => $dis:ident ) => {
            $crate::bind_raw!(
                &[$(Mod::$mod), *],
                Key::$keyt( $( $key ), * ),
                &[$(Flag::$flag), *],
                DispatchType::$dis
            )
        };
        ($( $mod:ident ) *,$keyt:ident,$( $key:expr ), * => $dis:ident, $( $arg:expr ), *) => {
            $crate::bind_raw!(
                &[$(Mod::$mod), *],
                Key::$keyt( $( $key ), * ),
                &[],
                DispatchType::$dis( $($arg),* )
            )
        };
        ($( $mod:ident ) *,$keyt:ident,$( $key:expr ), * => $dis:ident ) => {
            $crate::bind_raw!(
                &[$(Mod::$mod), *],
                Key::$keyt( $( $key ), * ),
                &[],
                DispatchType::$dis
            )
        };
    }
}

#[test]
fn test_binds() {
    use binds::*;
    let binding = Binding {
        mods: &[Mod::SUPER],
        key: Key::Key("v"),
        flags: &[],
        dispatcher: DispatchType::ToggleFloating(None),
    };
    let built_bind = match Binder::gen_str(binding) {
        Ok(v) => v,
        Err(e) => panic!("Error occured: {e}"), // Note to greppers: this is in a test!
    };
    assert_eq!(built_bind, "SUPER,v,togglefloating");
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod join_tests {
    use crate::config::binds::{Flag, Join as _};
    use crate::shared::Mod;

    /// Four byte-identical `Join` impls collapsed into one blanket impl. These
    /// pin the output for all four original receiver types so the refactor is
    /// behaviour-preserving, not merely compiling.
    #[test]
    fn join_is_unchanged_for_every_original_receiver() {
        let mods: Vec<Mod> = vec![Mod::SUPER, Mod::SHIFT];
        let flags: Vec<Flag> = vec![Flag::Locked, Flag::Repeat];

        assert_eq!(mods.join(), "SUPERSHIFT");
        assert_eq!(mods.as_slice().join(), "SUPERSHIFT");
        assert_eq!(flags.join(), "le");
        assert_eq!(flags.as_slice().join(), "le");
    }

    #[test]
    fn joining_nothing_yields_an_empty_string() {
        let mods: Vec<Mod> = vec![];
        let flags: &[Flag] = &[];
        assert_eq!(mods.join(), "");
        assert_eq!(flags.join(), "");
    }

    /// §2.10h renamed the variants, not the letters. This pins the entire
    /// vocabulary in declaration order, so a future rename cannot drift into a
    /// different generated bind: the letters are the ones upstream's own flag
    /// switch defines at `ConfigManager.cpp:1516-1529`.
    #[test]
    fn every_flag_variant_renders_its_upstream_letter() {
        let flags: Vec<Flag> = vec![
            Flag::Locked,
            Flag::Release,
            Flag::Repeat,
            Flag::NonConsuming,
            Flag::Mouse,
            Flag::Transparent,
            Flag::IgnoreMods,
            Flag::MultiKey,
            Flag::HasDescription,
            Flag::DontInhibit,
        ];
        assert_eq!(flags.join(), "lrenmtisdp");
    }
}
