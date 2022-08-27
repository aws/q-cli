Name: fig
Version: $VERSION
Release: $RELEASE
Summary: Fig for Linux
License: Fig License
Group: Applications/System
URL: https://fig.io
Source0: fig-%{version}-%{release}.tar.bz2
NoSource: 0

Requires: webkit2gtk3
Requires: gtk3
Requires: libappindicator-gtk3-devel
Requires: ibus

%description
%{summary}

%prep

%build
echo oOoOoOoO im buildin!

%install
cp -r %{_builddir}/fig-%{version}-%{release}/usr %{buildroot}

%clean
rm -rf %{buildroot}

%preun
fig _ uninstall-for-all-users

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

%changelog
