{ self }:

{ config, lib, pkgs, ... }:

let
  cfg = config.services.portproxy;

  configFile =
    if cfg.configPath != null then cfg.configPath
    else
      pkgs.writeText "portproxy.toml"
        (lib.concatMapStringsSep "\n" (p: ''
          [[proxy]]
          listen = "${p.listen}"
          connect = "${p.connect}"
        '') cfg.proxies);
in
{
  options.services.portproxy = {
    enable = lib.mkEnableOption "portproxy TCP port forwarding service";

    package = lib.mkOption {
      type = lib.types.package;
      default = self.packages.${pkgs.system}.default;
      defaultText = lib.literalExpression "the portproxy package built by this flake";
      description = "The portproxy package to use.";
    };

    proxies = lib.mkOption {
      type = lib.types.listOf (lib.types.submodule {
        options = {
          listen = lib.mkOption {
            type = lib.types.str;
            example = ":8080";
            description = "Address and port to listen on (e.g. \":8080\", \"127.0.0.1:8080\").";
          };
          connect = lib.mkOption {
            type = lib.types.str;
            example = "internal.lan:80";
            description = "Address and port to forward connections to.";
          };
        };
      });
      default = [ ];
      example = lib.literalExpression
        ''[ { listen = ":8080"; connect = "internal.lan:80"; } ]'';
      description = "List of proxy mappings to define inline. Generates a TOML config file automatically.";
    };

    configPath = lib.mkOption {
      type = lib.types.nullOr lib.types.path;
      default = null;
      example = "/etc/portproxy.toml";
      description = "Path to an external portproxy TOML config file. When set, overrides the ''proxies'' option.";
    };

    logLevel = lib.mkOption {
      type = lib.types.enum [ "error" "warn" "info" "debug" "trace" ];
      default = "info";
      description = "Log verbosity level.";
    };

    logDir = lib.mkOption {
      type = lib.types.nullOr lib.types.str;
      default = null;
      example = "/var/log/portproxy";
      description = "Directory to write log files to. File logging is disabled when null.";
    };

    openFirewall = lib.mkEnableOption "opening listen ports in the firewall";
  };

  config = lib.mkIf cfg.enable {
    assertions = [
      {
        assertion = cfg.proxies != [ ] || cfg.configPath != null;
        message = "services.portproxy: at least one of ''proxies'' or ''configPath'' must be set.";
      }
      {
        assertion = cfg.openFirewall -> cfg.configPath == null;
        message = "services.portproxy: ''openFirewall'' is not supported when using ''configPath'', because listen ports cannot be determined at evaluation time.";
      }
    ];

    networking.firewall.allowedTCPPorts =
      lib.mkIf cfg.openFirewall
        (map (p: lib.toInt (lib.last (lib.splitString ":" p.listen))) cfg.proxies);

    systemd.services.portproxy = {
      description = "portproxy TCP port forwarding";
      wantedBy = [ "multi-user.target" ];
      after = [ "network-online.target" ];
      wants = [ "network-online.target" ];

      serviceConfig = {
        ExecStart = lib.concatStringsSep " " (
          [
            "${cfg.package}/bin/portproxy"
            "--config-path ${configFile}"
            "--log-level ${cfg.logLevel}"
          ]
          ++ lib.optionals (cfg.logDir != null) [
            "--log-dir ${cfg.logDir}"
          ]
        );
        Restart = "on-failure";
      };
    };
  };
}
