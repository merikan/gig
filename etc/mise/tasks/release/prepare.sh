#!/usr/bin/env bash

#MISE description="prepare for a release"
#MISE dir="{{cwd}}"

set -euo pipefail

RELEASE_VERSION=""
DRY_RUN=false

usage() {
  cat <<EOF
  Usage: $0 [options] [release-version]

  Prepare for a release

  Paas a release version or let git-cliff calculate the version.


  Arguments:
    release          Release version in the format of [major.minor.patch]

  Options:
    -n, --dry-run    Show what would be done without making any changes.
    -h, --help       Show this help message.

EOF
}

info() { echo -e "\033[0;32m[INFO]\033[0m  $1"; }
warn() { echo -e "\033[1;33m[WARN]\033[0m  $1"; }
error() { echo -e "\033[0;31m[ERROR]\033[0m $1" >&2; }
die() {
  error "$*"
  exit 1
}

parse_args() {
  local positional_args=()

  while [[ $# -gt 0 ]]; do
    case "$1" in
    -h | --help)
      usage
      exit 0
      ;;
    -n | --dry-run)
      DRY_RUN=true
      shift 1
      ;;
    -*)
      error "Unknown option: $1"
      usage >&2
      exit 1
      ;;
    *)
      positional_args+=("$1")
      shift 1
      ;;
    esac
  done

  # Handle Positional Arguments
  local NO_OF_MANDATORY_ARGS=0
  local arg_count=${#positional_args[@]}
  if [[ "$arg_count" -lt $NO_OF_MANDATORY_ARGS ]]; then
    error "Incorrect number of arguments. Expected 1, got $arg_count." >&2
    usage >&2
    exit 1
  else
    if [[ "$arg_count" -ge 1 ]]; then
      RELEASE_VERSION="${positional_args[0]}"
    fi
  fi
}

prereq() {
  local tools=("git-cliff" "sed")
  for tool in "${tools[@]}"; do
    command -v "$tool" >/dev/null 2>&1 ||
      die "Required tool not found: $tool"
  done
}


# -----------------------------------------------------------------------------
# Check git for dirty directory
# -----------------------------------------------------------------------------
check_git() {
  if [ -n "$(git status --porcelain)" ]; then
    git status --short
    die "Working directory is not clean. Commit or stash your changes first."
  fi
}

# -----------------------------------------------------------------------------
# Prompt the user to confirm the new version passed as a
# script argument or calulated by git-cliff. Also gives the user the
# opportunity to type their own version.
#
# @global             RELEASE_VERSION
# @return             target version (stdout)
# -----------------------------------------------------------------------------
prompt_release_version() {

  local suggested_version="$RELEASE_VERSION"

  if [ -n "$suggested_version" ]; then
    info "Using provided argument for version suggestion: ${suggested_version}" >&2
  else
    local cliff_tag_version
    cliff_tag_version=$(git cliff --bumped-version)
    suggested_version="${cliff_tag_version#v}"
    if [ -z "$suggested_version" ]; then
      die "No new version bump detected by git-cliff (no relevant commits)."
    fi
    info "Suggested version bump from git-cliff: ${suggested_version}" >&2
  fi

  local target_version=""
  while [ -z "$target_version" ]; do
    read -rp $'\n'"Accept '${suggested_version}', type custom version, or 'n' to cancel [Y/n]: " USER_INPUT
    USER_INPUT=${USER_INPUT:-Y}

    case "$USER_INPUT" in
    [Nn]*)
      info "Release canceled." >&2
      exit 1
      ;;
    [Yy]*)
      target_version="$suggested_version"
      ;;
    *)
      local custom_version="$USER_INPUT"

      # Prompt for confirmation of the custom version
      read -rp "You entered '${custom_version}'. Are you sure? [y/N]: " CONFIRM_CUSTOM
      CONFIRM_CUSTOM=${CONFIRM_CUSTOM:-N}

      case "$CONFIRM_CUSTOM" in
      [Yy]*)
        target_version="$custom_version"
        ;;
      *)
        info "Custom version rejected. Please select again." >&2
        ;;
      esac
      ;;
    esac
  done

  info "Proceeding with release version: ${target_version}" >&2
  echo "$target_version"
}

# -----------------------------------------------------------------------------
# Creates a prepare for release commit
#
# @param new_version  The new version to use
# -----------------------------------------------------------------------------
prepare_release() {
  local new_version="$1"

  local tag_name="v$new_version"
  local tag_message="Release $tag_name"
  local commit_message="chore(release): prepare for release v$new_version"

  if $DRY_RUN; then
    warn "Dry run: no changes will be made."
  fi

  # update the changelog
  if $DRY_RUN; then
    info "[DRY-RUN] Would update CHANGELOG.md via: git-cliff --tag \"$new_version\""
    git-cliff --tag "$new_version" >&2
  else
    git-cliff --tag "$new_version"
    git add CHANGELOG.md
  fi

  # bump version
  if $DRY_RUN; then
    info "[DRY-RUN] Would bump version to \"$new_version\" in Cargo.toml"
  else
    sed -i -E "0,/^version = \".*\"/s//version = \"$new_version\"/" Cargo.toml
    local updated_version
    updated_version=$(sed -nE '/^\[package\]/,/^\[/ s/^version = "(.*)"/\1/p' Cargo.toml | head -n 1)

    if [ "$updated_version" = "$new_version" ]; then
      info "Version updated to $updated_version in Cargo.toml"
    else
      die "Verification failed. Expected $new_version, got $updated_version"
    fi
    # sync cargo.lock
    cargo check --quiet
    git add Cargo.toml Cargo.lock
  fi

  if $DRY_RUN; then
    info "[DRY-RUN] Would commit: $commit_message"
    info "[DRY-RUN] Would create tag: $tag_name with message '$tag_message'"
    info "Dry run complete. No changes were made."
  else
    git add -A && git commit -m "$commit_message"
    git tag -a "$tag_name" -m "Release $tag_name"

    read -rp "$\nPush commit and tag '$tag_name' to origin? [y/N] " CONFIRM
    if [[ "$CONFIRM" =~ ^[Yy]$ ]]; then
      # Get current branch name dynamically
      local current_branch
      current_branch=$(git rev-parse --abbrev-ref HEAD)

      info "Pushing commit and tag $tag_name to origin/$current_branch..."
      git push --atomic origin "$current_branch" "$tag_name"

      info "Successfully pushed release to origin."
    else
      info "Skipped push. The local commit and tag remain intact."
      info "When ready push it yourself with \"git push --atomic origin \"$current_branch\" \"$tag_name\"\""
    fi
  fi

}

main() {
  prereq
  parse_args "$@"
  check_git
  local new_version
  new_version=$(prompt_release_version)
  prepare_release "$new_version"
}

main "$@"
