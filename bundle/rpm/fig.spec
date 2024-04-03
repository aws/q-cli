Name: fig
Version: $VERSION
Release: $RELEASE
Summary: Fig for Linux
License: Fig License
Group: Applications/System
URL: https://fig.io
Conflicts: fig-minimal

Requires: webkit2gtk4.1
Requires: gtk3
Requires: libappindicator-gtk3-devel
Requires: ibus

# disable stripping
%define __strip /bin/true

%description
%{summary}

%install
rm -r %{buildroot}
BASE=%{_builddir}/fig-%{version}-%{release}
if [[ -d $BASE-1.$ARCH ]]; then
    cp -r $BASE-1.$ARCH/ %{buildroot}
else
    cp -r $BASE.$ARCH/ %{buildroot}
fi

%clean
rm -rf %{buildroot}

%posttrans
(ls /etc/yum.repos.d/fig.repo>/dev/null && sed -i 's/f$releasever\///' '/etc/yum.repos.d/fig.repo') || true

%files
/usr/bin/fig
/usr/bin/fig_desktop
/usr/bin/figterm
/usr/lib/systemd/user/fig.service
/usr/lib/environment.d/60-fig.conf
/usr/share/applications/fig.desktop
/usr/share/icons/hicolor/16x16/apps/fig.png
/usr/share/icons/hicolor/22x22/apps/fig.png
/usr/share/icons/hicolor/24x24/apps/fig.png
/usr/share/icons/hicolor/32x32/apps/fig.png
/usr/share/icons/hicolor/48x48/apps/fig.png
/usr/share/icons/hicolor/64x64/apps/fig.png
/usr/share/icons/hicolor/128x128/apps/fig.png
/usr/share/icons/hicolor/256x256/apps/fig.png
/usr/share/icons/hicolor/512x512/apps/fig.png
/usr/share/pixmaps/fig.png
/usr/share/fig/manifest.json
/usr/share/licenses/fig/LICENSE
