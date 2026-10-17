# dotinstall live: tty1 runs the boot flow, tty2 the text installer.
[ -f ~/.bashrc ] && . ~/.bashrc
case "$(tty)" in
  /dev/tty1) [ -z "${DOTINSTALL_FLOW_DONE:-}" ] && export DOTINSTALL_FLOW_DONE=1 && /usr/lib/dotinstall/live-flow ;;
  /dev/tty2) [ -z "${DOTINSTALL_FLOW_DONE:-}" ] && export DOTINSTALL_FLOW_DONE=1 && dotinstall-text ;;
esac
