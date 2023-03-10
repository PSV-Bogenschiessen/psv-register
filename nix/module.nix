service-name: service-pkg:
{ lib, config, pkgs, ... }:
with lib;
let
  cfg = config.services."${service-name}";
in
{
  options = {
    enable = mkEnableOption "${service-name} service";
    database-location = mkOption {
      type = types.str;
      default = "/var/psv-register/${service-name}.sqlite";
    };
    nginx = mkOption {
      description = lib.mkDoc ''
        Configuration for nginx reverse proxy.
      '';
      type = types.submodule {
        options = {
          enable = mkOption {
            type = types.bool;
            default = false;
            description = lib.mdDoc ''
              Configure the nginx reverse proxy settings.
            '';
          };

          hostName = mkOption {
            type = types.str;
            description = lib.mdDoc ''
              The hostname use to setup the virtualhost configuration
            '';
          };
        };
      };
    };
    smtp-password-file = mkOption {
      type = types.str;
      example = "/etc/passwords/smtp.password";
      description = ''
        Path to a file containing the password. This overwrites the password given in the settings.
        This option is mandatory because you shouldn't put the real password into the nix store (settings of this module).
      '';
    };
    settings = mkOption {
      type = pkgs.formats.toml.type;
      default = { };
      example = literalExpression ''
        {
          port = 3000;
          mail_server = {
            smtp_server = "smtp.mymail.com";
            smtp_username = "myuser";
            smtp_password = "t0p_secret";
          };
          mail_message = {
            sender_name = "Sender";
            sender_address = "me@mymail.com";
            subject = "Registration accepted";
          };
        }
      '';
      description = ''
        config.toml used for ${service-name}
      '';
    };
  };
  config = {
    systemd.services."${service-name}" = {
      wantedBy = [ "multi-user.target" ];
      serviceConfig.ExecStart = ''
        ${service-pkg}/bin/backend \
        --config-file ${pkgs.formats.toml.generate "${service-name} cfg.settings"} \
        --mail-template-file ${../backend/user_mail.tpl} \
        --database-file ${cfg.database-location} \
        --mail-password-file ${cfg.smtp-password-file}
      '';
    };

    services.nginx = {
      enable = mkDefault cfg.nginx.enable;
      recommendedProxySettings = mkDefault true;
      recommendedTlsSettings = mkDefault true;

      virtualHosts."${cfg.nginx.hostName}" = mkIf cfg.nginx.enable {
        enableACME = true;
        forceSSL = true;
        locations."/" = {
          proxyPass = "http://127.0.0.1:${cfg.settings.port}";
        };
      };
    };
  };
}
