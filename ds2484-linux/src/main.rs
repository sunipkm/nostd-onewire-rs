use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to I2C bus (e.g., /dev/i2c-1)
    #[arg(short, long)]
    path: String,
}

fn main() {
    // Initialize the logger
    env_logger::init();
    // Parse command line arguments
    let args = Args::parse();
    // Open the I2C bus
    let mut i2c = linux_embedded_hal::I2cdev::new(&args.path).expect("Failed to open I2C device");
    let mut delay = linux_embedded_hal::Delay;
    // Create a DS2484 instance
    let mut ds2484 = ds2484::Ds2484Builder::default()
        .build(&mut i2c, &mut delay)
        .expect("Failed to create DS2484 instance");
    // Create a DS28EA00 temperature sensor group
    let mut temp_sensors = ds28ea00::Ds28ea00Group::<16>::default()
        .with_resolution(ds28ea00::ReadoutResolution::Resolution12bit)
        .with_t_low(-40)
        .with_t_high(50)
        .with_toggle_pio(true);
    let mut delay = linux_embedded_hal::Delay;
    // Enumerate devices on the 1-Wire bus
    let devices = temp_sensors
        .enumerate(&mut ds2484)
        .expect("Failed to enumerate devices");
    log::info!("Found {} devices", devices);
    loop {
        // Trigger temperature conversion
        temp_sensors
            .trigger_temperature_conversion(&mut ds2484, &mut delay)
            .expect("Failed to trigger temperature conversion");
        // Read temperatures from the sensors
        for (rom, temp) in temp_sensors
            .read_temperatures(&mut ds2484, false)
            .expect("Failed to read temperatures")
        {
            log::info!("ROM: {:x}, Temperature: {}", rom, temp);
        }
    }
}
