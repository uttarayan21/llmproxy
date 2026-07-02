{
  config,
  lib,
  pkgs,
  llmproxy,
  ...
}:
with lib; let
  cfg = config.services.llmproxy;

  configFile = (pkgs.formats.toml {}).generate "llmproxy-config.toml" {
    server = {
      host = cfg.server.host;
      port = cfg.server.port;
    };

    database = {
      url = cfg.database.url;
    };

    auth = {
      enable_session = cfg.auth.enableSession;
      enable_header = cfg.auth.enableReverseProxyAuth;
      header_name = cfg.auth.reverseProxyHeaderName;
      disable_all = cfg.auth.disableAll;
    };
  };
in {
  options.services.llmproxy = {
    enable = mkEnableOption "LLMPROXY observability platform for LLM APIs";

    package = mkOption {
      type = types.nullOr types.path;
      default = null;
      description = "Path to the LLMPROXY source directory. If null, uses the module's directory.";
    };

    user = mkOption {
      type = types.str;
      default = "llmproxy";
      description = "User account under which LLMPROXY runs.";
    };

    group = mkOption {
      type = types.str;
      default = "llmproxy";
      description = "Group under which LLMPROXY runs.";
    };

    server = {
      host = mkOption {
        type = types.str;
        default = "127.0.0.1";
        description = "Host address to bind to. Use 0.0.0.0 to bind to all interfaces.";
      };

      port = mkOption {
        type = types.port;
        default = 8080;
        description = "Port to listen on.";
      };
    };

    database = {
      url = mkOption {
        type = types.str;
        default = "sqlite:/var/lib/llmproxy/llmproxy.db";
        description = "Database connection URL. Defaults to SQLite in state directory.";
      };
    };

    auth = {
      enableSession = mkOption {
        type = types.bool;
        default = true;
        description = "Enable session-based authentication (login/register).";
      };

      enableReverseProxyAuth = mkOption {
        type = types.bool;
        default = true;
        description = "Enable header-based authentication (reverse proxy).";
      };

      reverseProxyHeaderName = mkOption {
        type = types.str;
        default = "Remote-User";
        description = "Header name to use for header-based authentication.";
      };

      # enableBasic = mkOption {
      #   type = types.bool;
      #   default = false;
      #   description = "Enable HTTP Basic Authentication.";
      # };

      disableAll = mkOption {
        type = types.bool;
        default = false;
        description = "Disable all authentication. WARNING: This is insecure!";
      };
    };

    environmentFile = mkOption {
      type = types.nullOr types.path;
      default = null;
      description = ''
        Environment file containing secrets. Can be used to override configuration
        with environment variables like DATABASE_URL, API keys, etc.
      '';
    };

    openFirewall = mkOption {
      type = types.bool;
      default = false;
      description = "Open the firewall port for LLMPROXY. Only enable if binding to a public interface.";
    };
  };

  config = mkIf cfg.enable {
    assertions = [
      {
        assertion = !cfg.auth.disableAll || cfg.server.host == "127.0.0.1";
        message = "When auth.disableAll is true, server.host should be 127.0.0.1 for security";
      }
    ];

    # Create user and group
    users.users.${cfg.user} = {
      isSystemUser = true;
      group = cfg.group;
      description = "LLMPROXY service user";
      home = "/var/lib/llmproxy";
      createHome = true;
    };

    users.groups.${cfg.group} = {};

    # Systemd service
    systemd.services.llmproxy = {
      description = "LLMPROXY - Observability platform for LLM APIs";
      after = ["network.target"];
      wantedBy = ["multi-user.target"];

      environment = {
        CONFIG_PATH = toString configFile;
        RUST_LOG = "backend=info,tower_http=info";
      };

      serviceConfig = {
        Type = "simple";
        User = cfg.user;
        Group = cfg.group;
        ExecStart = "${llmproxy}/bin/backend";
        Restart = "on-failure";
        RestartSec = "5s";

        # Security hardening
        NoNewPrivileges = true;
        PrivateTmp = true;
        ProtectSystem = "strict";
        ProtectHome = true;
        ReadWritePaths = ["/var/lib/llmproxy"];
        ProtectKernelTunables = true;
        ProtectKernelModules = true;
        ProtectControlGroups = true;
        RestrictAddressFamilies = ["AF_INET" "AF_INET6" "AF_UNIX"];
        RestrictNamespaces = true;
        LockPersonality = true;
        MemoryDenyWriteExecute = false; # Required for Rust
        RestrictRealtime = true;
        RestrictSUIDSGID = true;
        PrivateDevices = true;

        # Environment file for secrets
        EnvironmentFile = mkIf (cfg.environmentFile != null) cfg.environmentFile;
      };

      preStart = ''
        # Ensure database directory exists
        mkdir -p /var/lib/llmproxy
        chown ${cfg.user}:${cfg.group} /var/lib/llmproxy
      '';
    };

    # Open firewall port if requested
    networking.firewall.allowedTCPPorts = mkIf cfg.openFirewall [cfg.server.port];
  };
}
