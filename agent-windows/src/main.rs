mod assessment;
mod cpu;
mod device;
mod evidence;
mod os;
mod report;
mod storage;

fn main() {
    let device = device::collect();
    let os = os::collect();
    let cpu = cpu::collect();
    let storage = storage::collect();
    let evidence = evidence::collect();
    let assessment = assessment::assess();

    let report = report::render(&device, &os, &cpu, &storage, &evidence, &assessment);

    println!("{report}");
}
