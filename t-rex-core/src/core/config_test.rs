//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use core::config::read_config;
use core::config::ApplicationCfg;
use core::config::DEFAULT_CONFIG;

#[test]
fn test_load_config() {
    let config = read_config("../t-rex-service/src/test/example.toml");
    println!("{:#?}", config);
    let config: ApplicationCfg = config.expect("load_config returned Err");
    assert!(config.service.mvt.viewer);
    assert_eq!(
        config.datasource[0].dbconn,
        Some("postgresql://postgres@127.0.0.1/osm_buildings".to_string())
    );
    assert_eq!(config.grid.predefined, Some("web_mercator".to_string()));
    assert_eq!(config.tilesets.len(), 1);
    assert_eq!(config.tilesets[0].name, "osm");
    assert_eq!(config.tilesets[0].layers.len(), 3);
    assert_eq!(config.tilesets[0].layers[0].name, "points");
    assert!(config.cache.is_none());
    assert_eq!(config.webserver.port, Some(8080));
}

#[test]
fn test_parse_error() {
    let config: Result<ApplicationCfg, _> = read_config("src/core/mod.rs");
    assert_eq!(
        "src/core/mod.rs - unexpected character found: `/` at line 1",
        config.err().unwrap()
    );

    let config: Result<ApplicationCfg, _> = read_config("wrongfile");
    assert_eq!("Could not find config file!", config.err().unwrap());
}

#[test]
fn test_default_config() {
    use core::parse_config;
    let config: ApplicationCfg = parse_config(DEFAULT_CONFIG.to_string(), "").unwrap();
    assert_eq!(config.webserver.port, Some(6767));
}

#[test]
fn test_envvar_expansion() {
    use core::parse_config;
    use std::env;

    let toml = r#"
        [service.mvt]
        viewer = true

        [[datasource]]
        dbconn = "${MYDBCONN}"

        [grid]
        predefined = "web_mercator"

        [[tileset]]
        name = "ts${MYPORT}"
        minzoom = 0
        maxzoom = 22

        [[tileset.layer]]
        name = "layer ${ MYPORT }"

        [webserver]
        bind = "127.0.0.1"
        port = ${MYPORT}
        "#;

    let config: Result<ApplicationCfg, _> = parse_config(toml.to_string(), "");
    assert_eq!(
        r#"Environment variable `MYDBCONN` undefined"#,
        config.err().unwrap()
    );

    env::set_var("MYDBCONN", "postgresql://pi@localhost/geostat");
    env::set_var("MYPORT", "9999");
    let config: ApplicationCfg =
        parse_config(toml.to_string(), "").expect("parse_config returned Err");
    assert_eq!(
        config.datasource[0].dbconn,
        Some("postgresql://pi@localhost/geostat".to_string())
    );
    assert_eq!(&config.tilesets[0].name, "ts9999");
    assert_eq!(&config.tilesets[0].layers[0].name, "layer ${ MYPORT }");
    assert_eq!(config.webserver.port, Some(9999));
}

#[test]
fn test_missing_geometry_field() {
    use core::parse_config;

    let toml = r#"
        [service.mvt]
        viewer = true

        [[datasource]]
        dbconn = "postgresql://user:pass@host/database"

        [grid]
        predefined = "web_mercator"

        [[tileset]]
        name = "points"

        [[tileset.layer]]
        name = "points"
        table_name = "mytable"
        #MISSING: geometry_field = "wkb_geometry"
        geometry_type = "POINT"

        [webserver]
        bind = "127.0.0.1"
        port = 6767
        "#;
    let config: Result<ApplicationCfg, _> = parse_config(toml.to_string(), "");
    assert_eq!(None, config.err()); //TODO: we should issue an error!
}

#[test]
fn test_datasource_compatibility() {
    use core::parse_config;
    // datasource spec beforce 0.8
    let toml = r#"
        [service.mvt]
        viewer = true

        [datasource]
        type = "postgis"
        url = "postgresql://pi@localhost/natural_earth_vectors"

        [grid]
        predefined = "web_mercator"

        [[tileset]]
        name = ""
        attribution = "© Contributeurs de OpenStreetMap" # Acknowledgment of ownership, authorship or copyright.

        [[tileset.layer]]
        name = ""

        [webserver]
        bind = "127.0.0.1"
        port = 6767
        threads = 4
        "#;
    let config: Result<ApplicationCfg, _> = parse_config(toml.to_string(), "");
    assert_eq!(
        " - invalid type: map, expected a sequence for key `datasource`",
        config.err().unwrap()
    );
    // let config: ApplicationCfg = config.expect("load_config returned Err");
    // assert_eq!(config.datasource[0].dbconn,
    //            Some("postgresql://pi@localhost/natural_earth_vectors".to_string()));
}
