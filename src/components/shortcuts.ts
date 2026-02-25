export type TabId = "dashboard";

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
    id: "dashboard-refresh",
    keys: "r",
    action: "Refresh dashboard or active course page",
    match: (input) => input === "r",
  },
  {
    id: "dashboard-open-finder",
    keys: "/",
    action: "Open course finder",
    match: (input) => input === "/",
  },
  {
    id: "dashboard-open-content-finder",
    keys: "f",
    action: "Open course content finder",
    match: (input) => input === "f",
  },
  {
    id: "dashboard-open-assignment-modal",
    keys: "Enter",
    action: "Open selected assignment details (course page)",
    match: (_input, key) => key.return,
  },
  {
    id: "dashboard-back",
    keys: "Esc",
    action: "Back to dashboard",
    match: (_input, key) => key.escape,
  },
  {
    id: "dashboard-up",
    keys: "Up",
    action: "Scroll up",
    match: (_input, key) => key.upArrow,
  },
  {
    id: "dashboard-down",
    keys: "Down",
    action: "Scroll down",
    match: (_input, key) => key.downArrow,
  },
  {
    id: "dashboard-expand",
    keys: "Right",
    action: "Expand selected node or move to first child",
    match: (_input, key) => key.rightArrow,
  },
  {
    id: "dashboard-collapse",
    keys: "Left",
    action: "Collapse selected node or move to parent",
    match: (_input, key) => key.leftArrow,
  },
  {
    id: "dashboard-page-up",
    keys: "PageUp",
    action: "Jump up by one page",
    match: (_input, key) => key.pageUp,
  },
  {
    id: "dashboard-page-down",
    keys: "PageDown",
    action: "Jump down by one page",
    match: (_input, key) => key.pageDown,
  },
  {
    id: "dashboard-home",
    keys: "Home",
    action: "Jump to top",
    match: (_input, key) => key.home,
  },
  {
    id: "dashboard-end",
    keys: "End",
    action: "Jump to bottom",
    match: (_input, key) => key.end,
  },
  {
    id: "course-finder-submit",
    keys: "Enter",
    action: "Open highlighted course",
    match: (_input, key) => key.return,
  },
  {
    id: "course-finder-cancel",
    keys: "Esc",
    action: "Close course finder",
    match: (_input, key) => key.escape,
  },
  {
    id: "course-content-finder-submit",
    keys: "Enter",
    action: "Jump to highlighted content",
    match: (_input, key) => key.return,
  },
  {
    id: "course-content-finder-cancel",
    keys: "Esc",
    action: "Close content finder",
    match: (_input, key) => key.escape,
  },
  {
    id: "assignment-modal-close",
    keys: "Esc",
    action: "Close assignment details modal",
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
      items: pick(["settings-open", "dashboard-refresh", "logout", "quit"]),
    },
    {
      title: "Settings Modal",
      items: pick(["settings-close"]),
    },
    {
      title: "Dashboard / Course Page",
      items: pick([
        "dashboard-open-finder",
        "dashboard-open-content-finder",
        "dashboard-open-assignment-modal",
        "dashboard-back",
        "dashboard-up",
        "dashboard-down",
        "dashboard-expand",
        "dashboard-collapse",
        "dashboard-page-up",
        "dashboard-page-down",
        "dashboard-home",
        "dashboard-end",
      ]),
    },
    {
      title: "Course Finder",
      items: pick(["course-finder-submit", "course-finder-cancel"]),
    },
    {
      title: "Course Content Finder",
      items: pick(["course-content-finder-submit", "course-content-finder-cancel"]),
    },
    {
      title: "Assignment Modal",
      items: pick(["assignment-modal-close"]),
    },
  ];
}
