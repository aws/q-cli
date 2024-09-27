import { InstallCheck, PlatformInfo } from "@/types/preferences";

/**
 * Checks if the given install check is valid for the given platform.
 *
 * @returns `true` if the install check is valid for `platformInfo`. `false` otherwise.
 */
export function isInstallCheckForPlatform(
  check: InstallCheck,
  platformInfo: PlatformInfo,
): boolean {
  if (!check.platformRestrictions) {
    return true;
  }

  return Object.entries(check.platformRestrictions)
    .map(
      ([key, value]) =>
        key in platformInfo &&
        value === platformInfo[key as keyof PlatformInfo],
    )
    .reduce((prev, curr, _) => prev && curr, true);
}
