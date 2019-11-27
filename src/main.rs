#![no_main]
#![no_std]

extern crate embedded_hal as hal;


//#[macro_use]
extern crate cortex_m_rt;

#[macro_use(block)]
extern crate nb;

use cortex_m_rt::{entry, exception};

extern crate cortex_m_semihosting as sh;
extern crate panic_semihosting;

extern crate tm4c123x_hal;

extern crate ds323x    ;

use tm4c123x_hal::sysctl::{self, SysctlExt};
use tm4c123x_hal::gpio::{AF2};
use tm4c123x_hal::gpio::GpioExt;
use tm4c123x_hal::time::U32Ext;
use tm4c123x_hal::timer::*;

use core::fmt::Write;

use hal::prelude::*;

extern crate smart_leds;
extern crate ws2812_spi;

use ws2812_spi as ws2812;

//use crate::ws2812::Ws2812;
//use crate::ws2812::prerendered::Timing;

use smart_leds::{colors, SmartLedsWrite};

use tm4c123x_hal::spi::{Spi};
pub use crate::hal::spi::{MODE_0, MODE_1, MODE_2, MODE_3};

use sh::hio;

#[entry]
fn main() -> ! {
    let mut stdout = hio::hstdout().unwrap();
    writeln!(stdout, "Hello, world!").unwrap();

    let p = tm4c123x_hal::Peripherals::take().unwrap();
    let mut sc = p.SYSCTL.constrain();

    sc.clock_setup.oscillator = sysctl::Oscillator::Main(
        sysctl::CrystalFrequency::_16mhz,
        sysctl::SystemClock::UsePll(sysctl::PllOutputFrequency::_80_00mhz),
    );
    let clocks = sc.clock_setup.freeze();

    //let mut delay = Delay::new(cp.SYST, &clocks);
    writeln!(stdout, "conf spi").unwrap();
    let mut portd = p.GPIO_PORTD.split(&sc.power_control);
    let spi = Spi::spi1(
        p.SSI1,
        (portd.pd0.into_af_push_pull::<AF2>(&mut portd.control), // SCK
         portd.pd2.into_af_push_pull::<AF2>(&mut portd.control), // Miso
         portd.pd3.into_af_pull_down::<AF2>(&mut portd.control)), // Mosi
        ws2812::MODE,
        3_000_000.hz(),
        &clocks,
        &sc.power_control
    );
    writeln!(stdout, "SPI configured").unwrap();

    writeln!(stdout, "Creating Timer").unwrap();

    let mut timer0 = Timer::wtimer0(p.WTIMER0,
                                   1.hz(),
                                   &sc.power_control,
                                   &clocks
    );

    writeln!(stdout, "Timer created").unwrap();
    writeln!(stdout, "Will block until timeout").unwrap();
    let _ret = block!(timer0.wait());
    writeln!(stdout, "Timeout").unwrap();
    // ws2812
    const MAX: usize = 6;

    //    let mut rendered_data = [0; MAX * 3 * 5];

    //    let mut ws = ws2812::prerendered::Ws2812::new(spi, Timing::new(3_000_000).unwrap(), &mut rendered_data);
    let mut ws = ws2812::Ws2812::new(spi);

    let mut data = [colors::BLUE, colors::GREEN,colors::GREEN,colors::GREEN,colors::GREEN,colors::GREEN ];
    let mut count = 0;
    let mut dir = true;

    loop {
        ws.write(data.iter().cloned()).unwrap();

        if ! dir {
            data.rotate_left(1);
        } else {
            data.rotate_right(1);
        }

        if count+2==MAX {
//            writeln!(stdout, "swap").unwrap();
            dir = !dir;
            count = 0;
        } else {
            count += 1;
        }
        let _ret = block!(timer0.wait());
    }
}

#[exception]
/// The hard fault handler
fn HardFault(ef: &cortex_m_rt::ExceptionFrame) -> ! {
    panic!("HardFault at {:#?}", ef);
}

#[exception]
/// The default exception handler
fn DefaultHandler(irqn: i16) {
    panic!("Unhandled exception (IRQn = {})", irqn);
}
