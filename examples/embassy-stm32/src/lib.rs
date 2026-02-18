#![no_std]

use core::format_args;
use embassy_executor::Spawner;
use embassy_net::{EthernetAddress, Stack, StackResources};
use embassy_stm32::eth::{Ethernet, GenericPhy, PacketQueue, Sma};
use embassy_stm32::peripherals::{ETH, ETH_SMA};
use embassy_stm32::rng::Rng;
use embassy_stm32::time::Hertz;
use embassy_stm32::{Config, Peripherals, bind_interrupts, eth, peripherals, rng};
use static_cell::StaticCell;
use veecle_os::osal::api::log::LogTarget;

bind_interrupts!(pub struct Irqs {
    ETH => eth::InterruptHandler;
    RNG => rng::InterruptHandler<peripherals::RNG>;
});

#[embassy_executor::task]
pub async fn net_task(
    mut runner: embassy_net::Runner<
        'static,
        Ethernet<'static, ETH, GenericPhy<Sma<'static, ETH_SMA>>>,
    >,
) -> ! {
    runner.run().await
}

pub fn initialize_networking(
    spawner: Spawner,
    peripherals: Peripherals,
    config: embassy_net::Config,
    mac_address: EthernetAddress,
) -> Stack<'static> {
    let mut rng = Rng::new(peripherals.RNG, Irqs);
    let mut seed = [0; 8];
    rng.fill_bytes(&mut seed);
    let seed = u64::from_le_bytes(seed);

    veecle_os::osal::embassy::log::Log::println(format_args!("Hello World!"));

    static PACKETS: StaticCell<PacketQueue<4, 4>> = StaticCell::new();
    let device = Ethernet::new(
        PACKETS.init(PacketQueue::<4, 4>::new()),
        peripherals.ETH,
        Irqs,
        peripherals.PA1,
        peripherals.PA7,
        peripherals.PC4,
        peripherals.PC5,
        peripherals.PG13,
        peripherals.PB13,
        peripherals.PG11,
        mac_address.0,
        peripherals.ETH_SMA,
        peripherals.PA2,
        peripherals.PC1,
    );

    // Init network stack
    static RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();

    let (stack, runner) =
        embassy_net::new(device, config, RESOURCES.init(StackResources::new()), seed);
    spawner.spawn(net_task(runner)).unwrap();
    stack
}

pub fn initialize_board() -> Peripherals {
    let mut global_config = Config::default();
    {
        use embassy_stm32::rcc::*;
        global_config.rcc.hse = Some(Hse {
            freq: Hertz(8_000_000),
            mode: HseMode::Bypass,
        });
        global_config.rcc.pll_src = PllSource::HSE;
        global_config.rcc.pll = Some(Pll {
            prediv: PllPreDiv::DIV4,
            mul: PllMul::MUL216,
            divp: Some(PllPDiv::DIV2), // 8mhz / 4 * 216 / 2 = 216Mhz
            divq: None,
            divr: None,
        });
        global_config.rcc.ahb_pre = AHBPrescaler::DIV1;
        global_config.rcc.apb1_pre = APBPrescaler::DIV4;
        global_config.rcc.apb2_pre = APBPrescaler::DIV2;
        global_config.rcc.sys = Sysclk::PLL1_P;
    }
    let peripherals = embassy_stm32::init(global_config);

    veecle_os::osal::embassy::log::Log::init();
    peripherals
}
