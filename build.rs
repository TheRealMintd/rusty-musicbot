use vergen::{vergen, Config, ShaKind};

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let mut config = Config::default();
	*config.build_mut().semver_mut() = false;
	*config.git_mut().branch_mut() = false;
	*config.git_mut().semver_dirty_mut() = Some("-dirty");
	*config.git_mut().sha_kind_mut() = ShaKind::Short;
	*config.rustc_mut().commit_date_mut() = false;
	*config.rustc_mut().host_triple_mut() = false;
	*config.rustc_mut().llvm_version_mut() = false;
	*config.rustc_mut().sha_mut() = false;

	Ok(vergen(config)?)
}
