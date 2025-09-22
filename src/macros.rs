/// Some macro foo that allows us to extract the configuration above and use it for different pin initializations
#[macro_export]
macro_rules! set_function_pins {
      // Defining Aliases
      (ALIASES, $pins:ident, $obj:ident, $(($id:ident, $pin:tt, $type:ident))*) => {
        build_pin_aliases!($obj, $(( $id, $pin))*);
      };

      // Loop Expansion
      ($func:ident, $pins:ident, $obj:ident, $( ($id:ident, $pin:tt, $type:ident) )* ) => {
          $(set_function_pins!(@each $func, $pins, $obj, $id, $pin, $type); )*
        };

      (@each ADC, $pins:ident, $obj:ident, $id:ident, $pin:literal, ADC) => {
        paste!($obj.[<set_ $id:lower>]($pins.[<gpio $pin>]));
      };

      (@each PWM, $pins:ident, $obj:ident, $id:ident, $pin:literal, PWM) => {
        paste!($obj.[<set_ $id:lower>]($pins.[<gpio $pin>]));
      };

      (@each I2C, $pins:ident, $obj:ident, $id:ident, $pin:literal, I2C) => {
        paste!(
          #[allow(non_snake_case)]
          let $id = $pins.[<gpio $pin>];
        );
      };

      (@each SPI, $pins:ident, $obj:ident, $id:ident, $pin:literal, SPI) => {
        paste!(
          #[allow(non_snake_case)]
          let $id = $pins.[<gpio $pin>];
        );
      };

      (@each UART, $pins:ident, $obj:ident, $id:ident, $pin:literal, UART) => {
        paste!(
          #[allow(non_snake_case)]
          let $id = $pins.[<gpio $pin>];
        );
      };

      (@each GPIO_IN, $pins:ident, $obj:ident, $id:ident, $pin:literal, IN) => {
        paste!(
          let pin: InputType = $pins.[<gpio $pin>].reconfigure().into_dyn_pin();
          $obj.add_pin(pin, $pin);
        );
      };

      (@each GPIO_OUT, $pins:ident, $obj:ident, $id:ident, $pin:literal, OUT) => {
        paste!(
          let pin: OutputType = $pins.[<gpio $pin>].reconfigure().into_dyn_pin();
          $obj.add_pin(pin, $pin);
        );
      };

      // Skip
      (@each $func:ident, $pins:ident, $obj:ident, $id:ident, $pin:tt, $type:ident ) => {};
    }

/// Build Global Pin ID Aliases. E.g. PinID::LED = 25 as usize.
/// Useful for referencing pin ids from aliases in the main program.
#[macro_export]
macro_rules! build_pin_aliases {
    // Entry
    ($obj:ident, $(($id:ident, $pin:tt))*) => {
        impl $obj {
            $(
                build_pin_aliases!(@each $id, $pin);
            )*
        }
    };
    // Match a literal pin
    (@each $id:ident, $pin:literal) => {
        pub const $id: u8 = $pin;
    };
    // Match "NA" (skip)
    (@each $id:ident, NA) => {};
}
