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
Requires: libappindicator-gtk3
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

%files
/usr/bin/fig
/usr/bin/fig_desktop
/usr/bin/fig_ibus_engine
"/usr/bin/zsh (figterm)"
"/usr/bin/bash (figterm)"
"/usr/bin/fish (figterm)"
/usr/bin/figterm
/usr/lib/systemd/user/fig.service
/usr/share/applications/fig.desktop
/usr/share/ibus/component/engine.xml
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
