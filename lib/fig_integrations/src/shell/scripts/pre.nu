mkdir ~/.fig/bin | ignore

def pathadd [path: string] {
  if not ($env.PATH | any $it == $path) {
    $env.PATH | prepend $path
  } else {
    $env.PATH
  }
}

let-env PATH = pathadd $"($env.HOME)/.fig/bin"
let-env PATH = pathadd $"($env.HOME)/.local/bin"

if "FIG_NEW_SESSION" in $env {
  let-env FIGTERM_SESSION_ID = $nothing
  let-env FIG_TERM = $nothing
  let-env FIG_ENV_VAR = $nothing
  let-env FIG_NEW_SESSION = $nothing
}

if "FIG_SET_PARENT_CHECK" not-in $env {
  if "FIG_SET_PARENT" not-in $env && "LC_FIG_SET_PARENT" in $env {
    let-env FIG_SET_PARENT = $env.LC_FIG_SET_PARENT
    let-env LC_FIG_SET_PARENT = $nothing
  }
  if "FIG_PARENT" not-in $env && "FIG_SET_PARENT" in $env {
    let-env FIG_PARENT = $env.FIG_SET_PARENT
    let-env FIG_SET_PARENT = $nothing
  }
  let-env FIG_SET_PARENT_CHECK = 1
}


let result = (^fig _ should-figterm-launch | complete)
let-env SHOULD_FIGTERM_LAUNCH = $result.exit_code

let should_launch = (
    ("PROCESS_LAUNCHED_BY_FIG" not-in $env || ($env.PROCESS_LAUNCHED_BY_FIG | str length) == 0)
    && ($env.SHOULD_FIGTERM_LAUNCH == 0 ||
       ($env.SHOULD_FIGTERM_LAUNCH == 2 && "FIG_TERM" not-in $env))
)

if $should_launch {
  let fig_shell = (fig _ get-shell | complete).stdout
  
  let fig_term_name = "nu (figterm)"
  let figterm_path = if ([$env.HOME ".fig" "bin" $fig_term_name] | path join | path exists) {
    [$env.HOME ".fig" "bin" $fig_term_name] | path join
  } else if (which figterm | length) > 0 {
    which figterm | first | get path
  } else {
    [$env.HOME ".fig" "bin" "figterm"] | path join
  }

  with-env {
    FIG_SHELL: $fig_shell
  } {
    exec $figterm_path
  }
}
