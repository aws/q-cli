Name: fig-headless
Version: $VERSION
Release: 1
Buildarch: $ARCH
Summary: Fig for Linux
License: Fig License
Group: Applications/System
URL: https://fig.io

%description
%{summary}

%install
rm -r %{buildroot}
cp -r %{_builddir}/fig-%{version}-%{release}.$ARCH/ %{buildroot}

%clean
rm -rf %{buildroot}

%preun
fig _ uninstall-for-all-users

%files
/usr/bin/fig
"/usr/bin/zsh (figterm)"
"/usr/bin/bash (figterm)"
"/usr/bin/fish (figterm)"
/usr/bin/figterm
/usr/lib/systemd/user/fig.service
/usr/share/fig/manifest.json

%changelog
