Name: fig-headless
Version: $VERSION
Release: 1
Summary: Fig for Linux
License: Fig License
Group: Applications/System
URL: https://fig.io
Conflicts: fig

# disable stripping
%define __strip /bin/true

%description
%{summary}

%install
rm -r %{buildroot}
cp -r %{_builddir}/fig-%{version}-%{release}.$ARCH/ %{buildroot}

%clean
rm -rf %{buildroot}

%preun
fig _ uninstall-for-all-users

%posttrans
(ls /etc/yum.repos.d/fig.repo>/dev/null && sed -i 's/f$releasever\///' '/etc/yum.repos.d/fig.repo') || true

%files
/usr/bin/fig
"/usr/bin/zsh (figterm)"
"/usr/bin/bash (figterm)"
"/usr/bin/fish (figterm)"
/usr/bin/figterm
/usr/share/fig/manifest.json
/usr/share/licenses/fig/LICENSE

%changelog
