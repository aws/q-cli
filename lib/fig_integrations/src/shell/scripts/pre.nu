mkdir ~/.local/bin | ignore

def pathadd [path: string] {
  if not ($env.PATH | any {|it| $it == $path }) {
    $env.PATH | prepend $path
  } else {
    $env.PATH
  }
}

let-env PATH = pathadd $"($env.HOME)/.local/bin"
let-env PATH = pathadd $"($env.HOME)/.local/bin"

if "CW_NEW_SESSION" in $env {
  let-env CWTERM_SESSION_ID = $nothing
  let-env CW_TERM = $nothing
  let-env CW_NEW_SESSION = $nothing
}

if "CW_SET_PARENT_CHECK" not-in $env {
  if "CW_PARENT" not-in $env and "CW_SET_PARENT" in $env {
    let-env CW_PARENT = $env.CW_SET_PARENT
    let-env CW_SET_PARENT = $nothing
  }
  let-env CW_SET_PARENT_CHECK = 1
}


let result = (^cw _ should-figterm-launch | complete)
let-env SHOULD_CWTERM_LAUNCH = $result.exit_code

let should_launch = (
    ("PROCESS_LAUNCHED_BY_CW" not-in $env or ($env.PROCESS_LAUNCHED_BY_CW | str length) == 0)
    and ($env.SHOULD_CWTERM_LAUNCH == 0 or
       ($env.SHOULD_CWTERM_LAUNCH == 2 and "CW_TERM" not-in $env))
)

if $should_launch {
  let CW_SHELL = (cw _ get-shell | complete).stdout
  
  let fig_term_name = "nu (figterm)"
  let figterm_path = if ([$env.HOME ".fig" "bin" $fig_term_name] | path join | path exists) {
    [$env.HOME ".fig" "bin" $fig_term_name] | path join
  } else if (which figterm | length) > 0 {
    which figterm | first | get path
  } else {
    [$env.HOME ".fig" "bin" "figterm"] | path join
  }

  with-env {
    CW_SHELL: $CW_SHELL
  } {
    exec $figterm_path
  }
}
