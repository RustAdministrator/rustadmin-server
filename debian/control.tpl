Source: rustadmin-server
Section: net
Priority: optional
Maintainer: RustAdministrator <rustadministrator@users.noreply.github.com>
Build-Depends: debhelper (>= 10), pkg-config
Standards-Version: 4.5.0
Homepage: https://github.com/rustadministrator/rustadmin-server/

Package: rustadmin-server-hbbs
Architecture: {{ ARCH }}
Depends: systemd ${misc:Depends}
Description: RustAdmin server
 Self-host your own RustAdmin server, it is free and open source.

Package: rustadmin-server-hbbr
Architecture: {{ ARCH }}
Depends: systemd ${misc:Depends}
Description: RustAdmin server
 Self-host your own RustAdmin server, it is free and open source.
 This package contains the RustAdmin relay server.

Package: rustadmin-server-utils
Architecture: {{ ARCH }}
Depends: ${misc:Depends}
Description: RustAdmin server
 Self-host your own RustAdmin server, it is free and open source.
 This package contains the rustadmin-utils binary.
