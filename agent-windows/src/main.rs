mod assessment;
mod cpu;
mod device;
mod encryption;
mod evidence;
mod os;
mod pdem;
mod personal_data;
mod report;
mod storage;
mod user_profiles;
mod volume;

fn main() {
    let device = device::collect();
    let os = os::collect();
    let cpu = cpu::collect();
    let storage = storage::collect();
    let volumes = volume::collect();
    let encryption = encryption::collect();

    let user_profiles = user_profiles::collect();

    let mut profile_paths = user_profiles
        .profiles
        .iter()
        .filter(|profile| !profile.special && profile.path != "unknown")
        .map(|profile| profile.path.clone())
        .collect::<Vec<_>>();

    if user_profiles.current_profile != "unknown" {
        profile_paths.push(user_profiles.current_profile.clone());
    }

    profile_paths.sort();
    profile_paths.dedup();

    let volume_roots = volumes
        .iter()
        .filter(|volume| volume.drive_letter != "unknown")
        .map(|volume| format!("{}:\\", volume.drive_letter))
        .collect::<Vec<_>>();

    let personal_data = personal_data::collect(&profile_paths, &volume_roots);

    let pdem = pdem::build(&personal_data);

    let evidence = evidence::collect();
    let assessment = assessment::assess();

    let a6 = report::A6Evidence {
        volumes: &volumes,
        encryption: &encryption,
    };

    let a7 = report::A7Evidence {
        user_profiles: &user_profiles,
        personal_data: &personal_data,
        pdem: &pdem,
    };

    let context = report::ReportContext {
        a6: &a6,
        a7: &a7,
        evidence: &evidence,
        assessment: &assessment,
    };

    let report = report::render(&device, &os, &cpu, &storage, &context);

    println!("{report}");
}
