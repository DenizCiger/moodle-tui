export type TabId = "courses";

export interface InputKey {
  upArrow: boolean;
  downArrow: boolean;
  leftArrow: boolean;
  rightArrow: boolean;
  pageDown: boolean;
  pageUp: boolean;
  home: boolean;
  end: boolean;
  return: boolean;
  escape: boolean;
  ctrl: boolean;
  shift: boolean;
  tab: boolean;
  backspace: boolean;
  delete: boolean;
  meta: boolean;
}

interface ShortcutDefinition {
  id: string;
  keys: string;
  action: string;
  match: (input: string, key: InputKey) => boolean;
}

export interface ShortcutSection {
  title: string;
  items: Array<Pick<ShortcutDefinition, "id" | "keys" | "action">>;
}

const SHORTCUTS: ShortcutDefinition[] = [
  {
    id: "settings-open",
    keys: "?",
    action: "Open settings/help",
    match: (input) => input === "?",
  },
  {
    id: "settings-close",
    keys: "Esc or ?",
    action: "Close settings/help",
    match: (input, key) => key.escape || input === "?",
  },
  {
    id: "quit",
    keys: "q",
    action: "Quit app",
    match: (input) => input === "q",
  },
  {
    id: "logout",
    keys: "l",
    action: "Logout and clear saved credentials",
    match: (input) => input === "l",
  },
  {
    id: "courses-refresh",
    keys: "r",
    action: "Refresh course list / active course page",
    match: (input) => input === "r",
  },
  {
    id: "courses-open",
    keys: "Enter",
    action: "Open selected course page",
    match: (_input, key) => key.return,
  },
  {
    id: "course-back",
    keys: "Esc",
    action: "Back to course list",
    match: (_input, key) => key.escape,
  },
  {
    id: "courses-up",
    keys: "Up",
    action: "Move selection / scroll up",
    match: (_input, key) => key.upArrow,
  },
  {
    id: "courses-down",
    keys: "Down",
    action: "Move selection / scroll down",
    match: (_input, key) => key.downArrow,
  },
  {
    id: "courses-page-up",
    keys: "PageUp",
    action: "Jump up by one page",
    match: (_input, key) => key.pageUp,
  },
  {
    id: "courses-page-down",
    keys: "PageDown",
    action: "Jump down by one page",
    match: (_input, key) => key.pageDown,
  },
  {
    id: "courses-home",
    keys: "Home",
    action: "Jump to first course",
    match: (_input, key) => key.home,
  },
  {
    id: "courses-end",
    keys: "End",
    action: "Jump to last course",
    match: (_input, key) => key.end,
  },
  {
    id: "courses-search",
    keys: "/",
    action: "Open course search",
    match: (input) => input === "/",
  },
  {
    id: "courses-search-clear",
    keys: "c",
    action: "Clear course search",
    match: (input) => input === "c",
  },
  {
    id: "courses-search-submit",
    keys: "Enter",
    action: "Close search input",
    match: (_input, key) => key.return,
  },
  {
    id: "courses-search-cancel",
    keys: "Esc",
    action: "Cancel search edit",
    match: (_input, key) => key.escape,
  },
];

const SHORTCUT_BY_ID = new Map(SHORTCUTS.map((shortcut) => [shortcut.id, shortcut]));

export function isShortcutPressed(id: string, input: string, key: InputKey): boolean {
  const shortcut = SHORTCUT_BY_ID.get(id);
  if (!shortcut) return false;
  return shortcut.match(input, key);
}

function pick(ids: string[]): Array<Pick<ShortcutDefinition, "id" | "keys" | "action">> {
  return ids
    .map((id) => SHORTCUT_BY_ID.get(id))
    .filter((shortcut): shortcut is ShortcutDefinition => Boolean(shortcut))
    .map(({ id, keys, action }) => ({ id, keys, action }));
}

export function getShortcutSections(_activeTab: TabId): ShortcutSection[] {
  return [
    {
      title: "Global",
      items: pick(["settings-open", "courses-refresh", "logout", "quit"]),
    },
    {
      title: "Settings Modal",
      items: pick(["settings-close"]),
    },
    {
      title: "Courses",
      items: pick([
        "courses-open",
        "course-back",
        "courses-up",
        "courses-down",
        "courses-page-up",
        "courses-page-down",
        "courses-home",
        "courses-end",
        "courses-search",
        "courses-search-clear",
      ]),
    },
    {
      title: "Course Search Input",
      items: pick([
        "courses-search-submit",
        "courses-search-cancel",
      ]),
    },
  ];
}
