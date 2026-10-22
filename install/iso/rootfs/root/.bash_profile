# KubaOS live: tty1 runs the boot flow, tty2 Kōbō's text mode.
[ -f ~/.bashrc ] && . ~/.bashrc
case "$(tty)" in
  /dev/tty1) [ -z "${KUBAOS_FLOW_DONE:-}" ] && export KUBAOS_FLOW_DONE=1 && /usr/lib/kubaos-live/live-flow ;;
  /dev/tty2) [ -z "${KUBAOS_FLOW_DONE:-}" ] && export KUBAOS_FLOW_DONE=1 && kobo-text ;;
esac
