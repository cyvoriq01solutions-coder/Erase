mod assessment;
mod cpu;
mod device;
mod encryption;
mod evidence;
mod os;
mod report;
mod storage;
mod volume;

fn main() {
    let device = device::collect();
    let os = os::collect();
    let cpu = cpu::collect();
    let storage = storage::collect();
    let volumes = volume::collect();
    let encryption = encryption::collect();
    let evidence = evidence::collect();
    let assessment = assessment::assess();

    let a6 = report::A6Evidence {
        volumes: &volumes,
        encryption: &encryption,
    };

    let report = report::render(&device, &os, &cpu, &storage, &a6, &evidence, &assessment);

    println!("{report}");
}
