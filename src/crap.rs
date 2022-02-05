
    let (package_name, package_vals) = dep;
    let path_to_dep = format!("ui/deps/{}", package_name);
    if Path::new(&path_to_dep).is_dir() {
        // if the dependency is already there
        if package_vals.is_str() {
            // fetch from registry
            // let _dep_config: DepConfigString = toml::from_str().unwrap();
            panic!("Ship does not support a central registry yet");
        } else {
            // fetch from git use options table
            for dep_option in package_vals.as_table().unwrap() {
                let parsed_dep_options: DepOptionTable = toml::from_str(
                    &(dep_option.0.to_owned() + " = \"" + dep_option.1.as_str().unwrap() + "\""),
                )
                .unwrap();
                if let Some(_git) = &parsed_dep_options.git {
                    // If the folder exists
                    let repo = git2::Repository::open(format!("ui/deps/{}", package_name)).unwrap();
                    if let Ok(head) = repo.head() {
                        if let Some(_local_branch_ref) = head.name() {
                            dbg!("At the point where we need to check if local is up to date");
                        };
                    };
                } else {
                    // if there is a path specified handle the copying of th path
                }
                if let Some(_version) = &parsed_dep_options.version {
                    // unwrap because above we just cloned it in
                    let _repo =
                        git2::Repository::open(format!("ui/deps/{}", package_name)).unwrap();
                    // TODO check if the version of the now copied repo is at the correct version
                    // checkout tag
                }

                println!("{:#?}", parsed_dep_options);
            }
            // println!("{:#?}, {:#?}", package_name, dep_config);
        }
    } else {
        // if the folder doesn't already exist
        for dep_option in package_vals.as_table().unwrap() {
            let parsed_dep_options: DepOptionTable = toml::from_str(
                &(dep_option.0.to_owned() + " = \"" + dep_option.1.as_str().unwrap() + "\""),
            )
            .unwrap();
            if let Some(git) = &parsed_dep_options.git {
                let re = Regex::new(r".com/").unwrap();
                let git = re.replace_all(git, r".com:");
                if let Err(e) = clone(&format!("git@{}", git), Path::new(&path_to_dep)) {
                    println!("{}", e);
                }
            }
        }
    }
