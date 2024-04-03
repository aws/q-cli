Name: fig-minimal
Version: $VERSION
Release: $RELEASE
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
/usr/bin/figterm
/usr/share/fig/manifest.json
/usr/share/licenses/fig/LICENSE

%changelog
