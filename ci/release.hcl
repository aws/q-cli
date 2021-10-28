notarize {
  path = "/Export/Fig.app"
  bundle_id = "com.mschrage.fig"
  staple = true
}

apple_id {
  username = "@env:NOTARIZE_USERNAME"
  password = "@env:NOTARIZE_PASSWORD"
}

