Name: escluse-agent
Version: 0.1.0
Release: 1%{?dist}
Summary: Escluse Agent for game server management
License: MIT
URL: https://escluse.com
Group: System Environment/Daemons
BuildArch: x86_64

%description
Escluse Agent connects your server to the Escluse platform, enabling
game server management through a web control panel.

%install
mkdir -p %{buildroot}/usr/local/bin
install -m 755 %{_sourcedir}/escluse-agent %{buildroot}/usr/local/bin/escluse-agent

%files
/usr/local/bin/escluse-agent

%post
chmod 755 /usr/local/bin/escluse-agent

%changelog
* Tue May 27 2026 Escluse Team <team@escluse.com> - 0.1.0
- Initial package
