/// Sets the /var/tmp/fig directory to be 0o1777 and user socket dir to be 0o700
///
/// We should probably migrate to some other dir because currently the first user
/// to create the socket will own it and other accounts are technically vulnerable
/// to that user
pub fn set_sockets_dir_permissions() -> Result<(), crate::Error> {
    use std::os::unix::prelude::PermissionsExt;
    let root_socket_dir = fig_util::directories::root_socket_dir();
    let user_socket_dir = fig_util::directories::sockets_dir()?;

    let meta = root_socket_dir.metadata()?;
    let mut perms = meta.permissions();
    if perms.mode() != 0o1777 {
        perms.set_mode(0o1777);
        std::fs::set_permissions(root_socket_dir, perms)?;
    }

    let meta = user_socket_dir.metadata()?;
    let mut perms = meta.permissions();
    if perms.mode() != 0o700 {
        perms.set_mode(0o700);
        std::fs::set_permissions(user_socket_dir, perms)?;
    }

    Ok(())
}
