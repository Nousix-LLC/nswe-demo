//! Two-way form binding primitives.
//!
//! [`FormField<T>`] holds the raw string value typed by the user and parses it to `T` on demand.
//! [`Bind<T>`] wraps a [`FormField<T>`] and produces the attribute and event-handler pair that
//! wires an HTML `<input>` element back to the field's reactive state.
//!
//! # Design — unvalidated input is never silently coerced
//!
//! The raw string is stored and exposed separately from the parsed value. A form submit handler
//! reads [`FormField::parsed`] and receives [`Err`] if the input is invalid, forcing explicit
//! error handling. No silent `unwrap` or default is applied.
//!
//! # Reactivity
//!
//! Both [`FormField::raw`] and [`FormField::parsed`] read the underlying `Signal<String>`.
//! Any [`create_effect`] or [`create_memo`] that calls them will re-run automatically whenever
//! [`FormField::set_raw`] is called — the full fine-grained reactive contract is preserved.
//!
//! [`create_effect`]: crate::signal::create_effect
//! [`create_memo`]: crate::signal::create_memo

use std::marker::PhantomData;
use std::rc::Rc;
use std::str::FromStr;

use crate::signal::{create_signal, Signal};
use crate::vdom::EventHandler;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use web_sys::{Event, HtmlInputElement};

/// Typed form state wrapper.
///
/// Keeps the raw string buffer of a form field separate from its parsed-and-validated form.
/// A form submit handler calls [`parsed`] and must handle the [`Err`] case — there is no path
/// by which unvalidated input is silently promoted to `T`.
///
/// # Reactivity
///
/// [`raw`] and [`parsed`] both read the internal `Signal<String>`. Calling either inside a
/// reactive context (an effect or a memo) subscribes that context to future changes so it
/// re-runs whenever [`set_raw`] is called.
///
/// [`raw`]: FormField::raw
/// [`parsed`]: FormField::parsed
/// [`set_raw`]: FormField::set_raw
pub struct FormField<T> {
    /// The reactive raw string backing this field.
    pub(crate) raw: Signal<String>,
    _marker: PhantomData<T>,
}

impl<T: FromStr + Clone + 'static> FormField<T>
where
    T::Err: std::fmt::Display,
{
    /// Create a field with the given initial raw string.
    ///
    /// The string is stored as-is; no initial parse attempt is made.
    pub fn new(initial: &str) -> Self {
        FormField {
            raw: create_signal(initial.to_string()),
            _marker: PhantomData,
        }
    }

    /// Return the current raw string value.
    ///
    /// Reading this inside a reactive context subscribes that context to changes from
    /// [`set_raw`].
    ///
    /// [`set_raw`]: FormField::set_raw
    pub fn raw(&self) -> String {
        self.raw.get()
    }

    /// Attempt to parse the raw value as `T`.
    ///
    /// Returns `Err(message)` where `message` is the [`Display`] representation of the parse
    /// error. The caller must handle the error; no silent coercion or default is applied.
    ///
    /// Reading this inside a reactive context subscribes that context to changes from
    /// [`set_raw`] (the dependency is on the underlying raw signal).
    ///
    /// [`Display`]: std::fmt::Display
    /// [`set_raw`]: FormField::set_raw
    pub fn parsed(&self) -> Result<T, String> {
        self.raw.get().parse::<T>().map_err(|e| e.to_string())
    }

    /// Update the raw string value.
    ///
    /// Notifies all reactive subscribers (effects and memos) that have read this field's
    /// [`raw`] or [`parsed`] value.
    ///
    /// [`raw`]: FormField::raw
    /// [`parsed`]: FormField::parsed
    pub fn set_raw(&self, raw: String) {
        self.raw.set(raw);
    }
}

/// Two-way binding descriptor for a [`FormField`].
///
/// Produces the `value` attribute and an `on_input` event handler that together keep an HTML
/// `<input>` element and the [`FormField`] in sync.
///
/// # Testability — approach 2 (WASM-free)
///
/// [`EventHandler`] is `Rc<dyn Fn()>` and takes no arguments. Reading the DOM input element's
/// current value inside a zero-argument closure requires a WASM call. To keep the handler
/// testable in native Rust, `Bind` maintains a secondary *input buffer* [`Signal<String>`].
///
/// Usage:
/// 1. Write the new input value into the buffer via [`set_input`].
/// 2. Invoke the handler (from the DOM event dispatch or directly in tests).
/// 3. The handler reads the buffer and propagates the value to the [`FormField`].
///
/// In a WASM renderer, step 1 is performed by the DOM event plumbing (reading
/// `HtmlInputElement::value()` and calling [`set_input`]) before dispatching the handler.
/// In tests, step 1 is performed explicitly with [`set_input`].
///
/// [`Signal<String>`]: crate::signal::Signal
/// [`set_input`]: Bind::set_input
/// [`on_input_handler`]: Bind::on_input_handler
pub struct Bind<T: FromStr + Clone + 'static>
where
    T::Err: std::fmt::Display,
{
    /// Shared handle to the field's raw signal (used for `value_attr` and to update on input).
    raw: Signal<String>,
    /// Secondary buffer written by the DOM layer (or by [`set_input`] in tests) just before
    /// the input handler is dispatched.
    ///
    /// [`set_input`]: Bind::set_input
    input_buf: Signal<String>,
    _marker: PhantomData<T>,
}

impl<T: FromStr + Clone + 'static> Bind<T>
where
    T::Err: std::fmt::Display,
{
    /// Create a [`Bind`] wrapping the given [`FormField`].
    ///
    /// The input buffer is initialised with the field's current raw value.
    pub fn new(field: &FormField<T>) -> Self {
        Bind {
            raw: field.raw,
            input_buf: create_signal(field.raw.get()),
            _marker: PhantomData,
        }
    }

    /// Return the `("value", raw_string)` attribute pair for an `<input>` element.
    ///
    /// Pass the returned pair into the element's attribute list to reflect the field's current
    /// value in the DOM.
    pub fn value_attr(&self) -> (&'static str, String) {
        ("value", self.raw.get())
    }

    /// Return an [`EventHandler`] that propagates the pending input buffer value to the field.
    ///
    /// When invoked the handler reads the current value of the internal input buffer signal
    /// and writes it to the field's raw signal, triggering reactive updates on `raw()` and
    /// `parsed()` subscribers.
    ///
    /// **Call [`set_input`] with the new value before invoking the handler** (or before the
    /// DOM dispatches it).
    ///
    /// [`set_input`]: Bind::set_input
    pub fn on_input_handler(&self) -> EventHandler {
        let raw = self.raw;
        let buf = self.input_buf;
        EventHandler::NoArg(Rc::new(move || {
            raw.set(buf.get());
        }))
    }

    /// Write `value` into the input buffer.
    ///
    /// In a WASM renderer this is called with the DOM input element's current value just
    /// before the event dispatch. In tests this replaces the DOM event entirely, enabling
    /// fully native (non-WASM) test execution.
    pub fn set_input(&self, value: String) {
        self.input_buf.set(value);
    }

    /// Return an [`EventHandler`] that reads the DOM input element's current value and writes
    /// it directly to the field's raw signal (REQ-ASYNC-042).
    ///
    /// Unlike [`on_input_handler`], which requires [`set_input`] to be called first, this
    /// handler reads `HtmlInputElement::value()` directly from the DOM event's target,
    /// bypassing the secondary input buffer entirely.
    ///
    /// The closure captures the field's raw signal. When invoked with a `&dyn Any` that
    /// downcasts to `&web_sys::Event`:
    /// 1. `event.target()` retrieves the [`EventTarget`].
    /// 2. `dyn_into::<HtmlInputElement>()` casts to the concrete input type.
    /// 3. `.value()` reads the current DOM value.
    /// 4. `raw.set(value)` updates the field signal, notifying all reactive subscribers.
    ///
    /// If any step fails (target absent, or the target is not an `<input>` element), the
    /// closure returns without updating the signal — no panic.
    ///
    /// This method is gated to `wasm32` because it references `web_sys` DOM types that are
    /// not available on native targets.
    ///
    /// [`on_input_handler`]: Bind::on_input_handler
    /// [`set_input`]: Bind::set_input
    /// [`EventTarget`]: web_sys::EventTarget
    #[cfg(target_arch = "wasm32")]
    pub fn on_input_event_handler(&self) -> EventHandler {
        let raw = self.raw;
        EventHandler::WithEvent(Rc::new(move |any: &dyn std::any::Any| {
            if let Some(event) = any.downcast_ref::<Event>() {
                if let Some(target) = event.target() {
                    if let Ok(input_el) = target.dyn_into::<HtmlInputElement>() {
                        raw.set(input_el.value());
                    }
                }
            }
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    // ---- FormField ----------------------------------------------------------------

    #[test]
    fn form_field_initial_raw_round_trip() {
        let field: FormField<u32> = FormField::new("42");
        assert_eq!(field.raw(), "42");
    }

    #[test]
    fn form_field_initial_parsed_success() {
        let field: FormField<u32> = FormField::new("42");
        assert_eq!(field.parsed(), Ok(42u32));
    }

    #[test]
    fn form_field_initial_parsed_failure() {
        let field: FormField<u32> = FormField::new("not-a-number");
        assert!(field.parsed().is_err());
        // The error message must be non-empty (Display of the parse error).
        assert!(!field.parsed().unwrap_err().is_empty());
    }

    #[test]
    fn form_field_set_raw_updates_raw() {
        let field: FormField<u32> = FormField::new("0");
        field.set_raw("99".to_string());
        assert_eq!(field.raw(), "99");
    }

    #[test]
    fn form_field_set_raw_then_parsed_success() {
        let field: FormField<u32> = FormField::new("0");
        field.set_raw("7".to_string());
        assert_eq!(field.parsed(), Ok(7u32));
    }

    #[test]
    fn form_field_set_raw_bad_value_gives_err() {
        let field: FormField<u32> = FormField::new("42");
        field.set_raw("bad".to_string());
        assert!(field.parsed().is_err());
    }

    #[test]
    fn form_field_set_raw_valid_then_invalid_then_valid() {
        let field: FormField<u32> = FormField::new("1");
        assert_eq!(field.parsed(), Ok(1u32));
        field.set_raw("xyz".to_string());
        assert!(field.parsed().is_err());
        field.set_raw("100".to_string());
        assert_eq!(field.parsed(), Ok(100u32));
    }

    #[test]
    fn form_field_raw_is_reactive() {
        // An effect that reads the raw signal should re-run when set_raw() is called.
        let field: FormField<u32> = FormField::new("0");
        let run_count = Rc::new(Cell::new(0u32));
        let run_count_clone = run_count.clone();
        // `Signal<T>` is Copy — capture the key for use in the 'static closure.
        let raw_sig = field.raw;
        crate::signal::create_effect(move || {
            let _ = raw_sig.get();
            run_count_clone.set(run_count_clone.get() + 1);
        });
        // The effect runs once immediately on registration.
        assert_eq!(run_count.get(), 1);
        field.set_raw("5".to_string());
        assert_eq!(run_count.get(), 2);
        field.set_raw("10".to_string());
        assert_eq!(run_count.get(), 3);
    }

    #[test]
    fn form_field_parsed_subscribers_notified_after_set_raw() {
        // A memo that depends on the raw signal is re-evaluated after set_raw.
        let field: FormField<u32> = FormField::new("1");
        let raw_sig = field.raw;
        let parsed_memo = crate::signal::create_memo(move || raw_sig.get().parse::<u32>().ok());
        assert_eq!(parsed_memo.get(), Some(1u32));
        field.set_raw("77".to_string());
        assert_eq!(parsed_memo.get(), Some(77u32));
        field.set_raw("bad".to_string());
        assert_eq!(parsed_memo.get(), None);
    }

    // ---- Bind ---------------------------------------------------------------

    #[test]
    fn bind_value_attr_returns_current_raw() {
        let field: FormField<u32> = FormField::new("42");
        let bind = Bind::new(&field);
        let (name, val) = bind.value_attr();
        assert_eq!(name, "value");
        assert_eq!(val, "42");
    }

    #[test]
    fn bind_value_attr_reflects_set_raw() {
        let field: FormField<u32> = FormField::new("0");
        let bind = Bind::new(&field);
        field.set_raw("55".to_string());
        let (_, val) = bind.value_attr();
        assert_eq!(val, "55");
    }

    #[test]
    fn bind_on_input_handler_updates_field() {
        // Simulate a DOM input event: set buffer, fire handler, verify field updates.
        let field: FormField<u32> = FormField::new("0");
        let bind = Bind::new(&field);
        bind.set_input("42".to_string());
        let handler = bind.on_input_handler();
        if let EventHandler::NoArg(f) = &handler {
            f();
        } else {
            unreachable!()
        }
        assert_eq!(field.raw(), "42");
        assert_eq!(field.parsed(), Ok(42u32));
    }

    #[test]
    fn bind_on_input_handler_bad_value_propagates_to_field() {
        // Invalid input must propagate faithfully — the handler never filters or coerces.
        let field: FormField<u32> = FormField::new("1");
        let bind = Bind::new(&field);
        bind.set_input("not_a_number".to_string());
        let handler = bind.on_input_handler();
        if let EventHandler::NoArg(f) = &handler {
            f();
        } else {
            unreachable!()
        }
        assert_eq!(field.raw(), "not_a_number");
        assert!(field.parsed().is_err());
    }

    #[test]
    fn bind_on_input_handler_multiple_updates() {
        let field: FormField<u32> = FormField::new("0");
        let bind = Bind::new(&field);
        let handler = bind.on_input_handler();
        let f = if let EventHandler::NoArg(f) = &handler {
            f.clone()
        } else {
            unreachable!()
        };
        for (input, expected) in [("1", 1u32), ("22", 22), ("333", 333)] {
            bind.set_input(input.to_string());
            f();
            assert_eq!(field.parsed(), Ok(expected));
        }
    }

    #[test]
    fn bind_on_input_handler_triggers_reactive_effect() {
        // Verify that firing the handler notifies reactive subscribers of the field.
        let field: FormField<u32> = FormField::new("0");
        let run_count = Rc::new(Cell::new(0u32));
        let run_count_clone = run_count.clone();
        let raw_sig = field.raw;
        crate::signal::create_effect(move || {
            let _ = raw_sig.get();
            run_count_clone.set(run_count_clone.get() + 1);
        });
        // Effect fires once on registration.
        assert_eq!(run_count.get(), 1);

        let bind = Bind::new(&field);
        bind.set_input("5".to_string());
        if let EventHandler::NoArg(f) = bind.on_input_handler() {
            f();
        } else {
            unreachable!()
        }
        assert_eq!(run_count.get(), 2);

        bind.set_input("10".to_string());
        if let EventHandler::NoArg(f) = bind.on_input_handler() {
            f();
        } else {
            unreachable!()
        }
        assert_eq!(run_count.get(), 3);
    }

    #[test]
    fn form_field_works_with_f64() {
        // Use 1.5 (not an approximate mathematical constant) to avoid clippy::approx_constant.
        let field: FormField<f64> = FormField::new("1.5");
        let parsed = field.parsed().expect("1.5 should parse as f64");
        assert!((parsed - 1.5_f64).abs() < 1e-10);
        field.set_raw("bad".to_string());
        assert!(field.parsed().is_err());
    }

    #[test]
    fn form_field_works_with_string_type() {
        // String::from_str is infallible — parsed() must always return Ok.
        let field: FormField<String> = FormField::new("hello");
        assert_eq!(field.parsed(), Ok("hello".to_string()));
        field.set_raw("world".to_string());
        assert_eq!(field.parsed(), Ok("world".to_string()));
    }

    #[test]
    fn bind_new_initialises_input_buf_from_field_current_value() {
        // The input buffer should start at the field's current raw value, not empty.
        let field: FormField<u32> = FormField::new("99");
        let bind = Bind::new(&field);
        // Fire the handler without setting the buffer — it should propagate "99" (not corrupt).
        let handler = bind.on_input_handler();
        if let EventHandler::NoArg(f) = &handler {
            f();
        } else {
            unreachable!()
        }
        assert_eq!(field.raw(), "99");
    }
}

// =====================================================================================
// WASM integration test — exercises the full DOM→signal path for REQ-ASYNC-042.
//
// Compiles only for wasm32 and runs under `wasm-pack test --headless --firefox`.
// A real <input> element is inserted into the document, its value is set, a synthetic
// input event is dispatched, and the test asserts the field signal reflects the typed value.
// =====================================================================================

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use super::*;
    use crate::vdom::EventHandler;
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::JsCast;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    /// REQ-ASYNC-042: an `input` event on a bound `<input>` element must update the signal to
    /// the user-typed value.
    ///
    /// Test flow:
    /// 1. Create a real `<input>` element and attach it to the document body so that a
    ///    dispatched event carries a valid `target()`.
    /// 2. Set the element's value to `"hello"` (simulating a user keystroke).
    /// 3. Build a `Bind<String>` around a fresh `FormField<String>`.
    /// 4. Call `bind.on_input_event_handler()` to obtain the event-carrying handler.
    /// 5. Wire the handler to the element via a `wasm_bindgen::Closure`.
    /// 6. Dispatch a synthetic `input` event — `event.target()` resolves to the element.
    /// 7. Assert the field signal equals `"hello"`.
    #[wasm_bindgen_test]
    fn on_input_event_handler_updates_signal_from_dom_value() {
        let document = web_sys::window().unwrap().document().unwrap();

        // A real DOM element attached to the body so dispatched events carry a valid target().
        let input_el: web_sys::HtmlInputElement =
            document.create_element("input").unwrap().unchecked_into();
        document.body().unwrap().append_child(&input_el).unwrap();

        // Pre-set the element's value to what a user would have typed.
        input_el.set_value("hello");

        // Build a fresh Bind<String> and obtain the event-carrying handler (REQ-ASYNC-042).
        let field: FormField<String> = FormField::new("");
        let bind = Bind::new(&field);
        let handler = bind.on_input_event_handler();

        // Wire the handler to the element via a wasm_bindgen Closure.
        let closure = Closure::<dyn FnMut(web_sys::Event)>::new(move |event: web_sys::Event| {
            if let EventHandler::WithEvent(ref f) = handler {
                f(&event as &dyn std::any::Any);
            }
        });
        input_el
            .add_event_listener_with_callback("input", closure.as_ref().unchecked_ref())
            .unwrap();
        // `forget()` is acceptable in a test: the closure has test scope and the element is
        // removed at the end of the test.
        closure.forget();

        // Dispatch a synthetic input event — event.target() is set to input_el by the browser.
        let event = web_sys::Event::new("input").unwrap();
        input_el.dispatch_event(&event).unwrap();

        // The handler must have read HtmlInputElement::value() and written to the signal.
        assert_eq!(
            field.raw(),
            "hello",
            "REQ-ASYNC-042: signal must equal the DOM value after the input event"
        );

        // Clean up the DOM so other tests in the same run are not affected.
        document.body().unwrap().remove_child(&input_el).unwrap();
    }
}
