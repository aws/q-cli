if [ -z "$FIG_PTY_SHELL_VAR" ]
  if begin; status --is-interactive; and not functions -q -- fig_osc; and [ "$TERM" != linux ]; end
    function fig_osc; printf "\033]697;"; printf $argv; printf "\007"; end
    function fig_copy_fn; functions -e $argv[2]; functions -c $argv[1] $argv[2]; end
    function fig_fn_defined; test (functions $argv[1] | grep -vE '^ *(#|function |end$|$)' | wc -l) != 0; end

    function fig_wrap_prompt
      set -l last_status $status
      fig_osc StartPrompt

      sh -c "exit $last_status"
      printf "%b" (string join "\n" $argv)
      fig_osc EndPrompt

      sh -c "exit $last_status"
    end

    function fig_preexec --on-event fish_preexec
      fig_osc PreExec

      if fig_fn_defined fig_user_mode_prompt
        fig_copy_fn fig_user_mode_prompt fish_mode_prompt
      end

      if fig_fn_defined fig_user_right_prompt
        fig_copy_fn fig_user_right_prompt fish_right_prompt
      end

      fig_copy_fn fig_user_prompt fish_prompt

      set fig_has_set_prompt 0
    end

    function fig_precmd --on-event fish_prompt
      if [ $fig_has_set_prompt = 1 ]
        fig_preexec
      end

      fig_osc "Dir=%s" $PWD
      fig_osc "Shell=fish" $PWD

      if fig_fn_defined fish_mode_prompt
        fig_copy_fn fish_mode_prompt fig_user_mode_prompt
        function fish_mode_prompt; fig_wrap_prompt (fig_user_mode_prompt); end
      end

      if fig_fn_defined fish_right_prompt
        fig_copy_fn fish_right_prompt fig_user_right_prompt
        function fish_right_prompt; fig_wrap_prompt (fig_user_right_prompt); end
      end

      fig_copy_fn fish_prompt fig_user_prompt 
      function fish_prompt; fig_wrap_prompt (fig_user_prompt); fig_osc NewCmd; end

      set fig_has_set_prompt 1
    end

    set fig_has_set_prompt 0
  end
  set FIG_PTY_SHELL_VAR 1
end
