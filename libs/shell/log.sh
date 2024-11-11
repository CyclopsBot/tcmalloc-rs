#!/bin/env bash

COLOR_RESET='\033[0m'
COLOR_DEBUG='\033[0;36m'     # Cyan
COLOR_INFO='\033[0;32m'      # Green
COLOR_WARN='\033[0;33m'      # Yellow
COLOR_ERROR='\033[0;31m'     # Red
COLOR_TIMESTAMP='\033[1;90m' # Dark Black
LOGGER_FMT=${LOGGER_FMT:="[%Y-%m-%d %H:%M:%S]"}
LOGGER_LVL=${LOGGER_LVL:="info"}

function log() {
  action=$1 && shift
  case $action in
  debug) [[ $LOGGER_LVL =~ debug|info ]] && echo -e "${COLOR_TIMESTAMP}$(date "+${LOGGER_FMT}")${COLOR_RESET} ${COLOR_DEBUG}DEBUG${COLOR_RESET} $*" 1>&2 ;;
  info) [[ $LOGGER_LVL =~ debug|info ]] && echo -e "${COLOR_TIMESTAMP}$(date "+${LOGGER_FMT}")${COLOR_RESET} ${COLOR_INFO}INFO${COLOR_RESET} $*" 1>&2 ;;
  warn) [[ $LOGGER_LVL =~ debug|info|warn ]] && echo -e "${COLOR_TIMESTAMP}$(date "+${LOGGER_FMT}")${COLOR_RESET} ${COLOR_WARN}WARN${COLOR_RESET} $*" 1>&2 ;;
  error) [[ ! $LOGGER_LVL =~ none ]] && echo -e "${COLOR_TIMESTAMP}$(date "+${LOGGER_FMT}")${COLOR_RESET} ${COLOR_ERROR}ERROR${COLOR_RESET} $*" 1>&2 ;;
  esac
  true
}
