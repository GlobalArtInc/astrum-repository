# Astrum Repository web UI

React 19, TypeScript (strict), Vite, Radix Themes, React Router 7, TanStack Query, Biome, Vitest and
Playwright. The Rust server serves the built `dist/` (see `web.dir` / `ASTRUM_WEB_DIR`).

```sh
corepack pnpm install
ASTRUM_URL=http://127.0.0.1:18301 corepack pnpm dev   # Vite on :5173, /service /repository /v2 go to the server
corepack pnpm lint && corepack pnpm typecheck && corepack pnpm test && corepack pnpm i18n:check && corepack pnpm build
ASTRUM_BIN=/path/to/astrum corepack pnpm e2e           # needs a build and a PostgreSQL, see "End-to-end"
```

## Layout

- `src/app/`: bootstrap, router, `area.ts` (the registration types), `areas.ts` (the list of areas),
  `legacy.ts` (old hash addresses to paths).
- `src/shell/`: header, menu, drawer, search panel host, theme, 404.
- `src/auth/`: UI state, permissions, login page, guard, session watcher.
- `src/api/`: fetch client, errors, query helpers, types of shared REST JSON.
- `src/ui/`: the component layer; import from `../../ui`.
- `src/i18n/`: translation, language cookie, number/size/date formatting.
- `src/areas/<area>/`: one folder per area with its pages, components, `strings/` and tests.

## Adding an area

1. Create `src/areas/<area>/index.ts` exporting an `Area`:

   ```ts
   export const browseArea: Area = {
     id: 'browse',
     routes: [
       { path: 'browse', load: () => import('./BrowsePage'), need: { some: 'astrum:repository-view' } },
       { path: 'browse/:repository/*', load: () => import('./RepositoryPage') },
     ],
     menu: [{ id: 'browse', to: '/browse', label: 'browse.menu.label', icon: FolderOpen, group: 'main',
              order: 30, need: { some: 'astrum:repository-view' } }],
     settings: [],   // tiles: { id, section: 'repository' | 'security', to, title, description, icon, order, need }
     search: [],     // header search providers: { id, order, need, Results, submit }
     watchers: [],   // components the shell renders on every page, e.g. the onboarding redirect
   };
   ```

   Pages are default exports when loaded with `load` (one chunk per page), or components with `page`.
2. Add it to the list in `src/app/areas.ts` (one line).
3. Menu orders in use: dashboard 10, search 20, browse 30, upload 40, settings 900 (group `admin`).

Routes are relative paths under the shell. `requiresUser: true` sends anonymous callers to `/login`;
`need` hides the page (signed-in callers see "You cannot open this page", anonymous ones go to the login
page). Use `Page` (title, breadcrumbs, actions, description) as the root of every page: it also sets the
document title `<title> - Astrum Repository`.

## Texts

Every visible text lives in `src/areas/<area>/strings/en.json` and `ru.json` (the shell, ui and auth have
their own). Keys are `<area>.<page>.<name>` and must start with the area id. Plurals are objects with the
`Intl.PluralRules` categories of each language (`one/other` in English, `one/few/many/other` in Russian):

```json
{ "browse.list.count": { "one": "{count} repository", "other": "{count} repositories" } }
```

Use `const t = useT()`, `t('browse.list.title')`, `t('browse.list.count', { count })`, and
`t.rich('key', { name: <code>x</code> })` for markup. Write keys as string literals so `pnpm i18n:check`
can see them; it fails when en and ru differ (keys, plural forms, placeholders) or a used key is missing.
Russian texts of the old UI (`public/**/astrum-i18n/**/ru.json`) may be reused; English is written afresh.

## Permissions and API

- `useCan(need)` and `useSession()` (`user`, `anonymousAccess`, `state.formats`, `state.baseUrl`). A need
  is a permission string (implied by a grant, same rules as the server), `{ all: [...] }`, `{ any: [...] }`
  or `{ some: 'astrum:repository-view' }` (the caller holds anything under that prefix).
- `useApi<T>(path)` reads JSON (query key `['api', path]`); after a change call
  `useInvalidateApi()(prefix)`. `api.post/put/delete` send JSON with the CSRF header. Errors are `ApiError`
  (`status`, `message`, `fieldErrors` of a 400); `describeError(t, error)` turns them into text. Build
  paths with ``path`/service/rest/v1/repositories/${name}` `` and `withQuery`.
- A 401 raises the session dialog (or the login page when anonymous access is off).

## Components (`src/ui`)

`Page`, `DataTable` (sorting, client paging or "show more", server "load more", skeleton, empty and error
states, row links and actions, cards on phones), `useForm` with `TextField`, `NumberField`,
`SecretField` (`stored` keeps a server-side secret when left empty), `SelectField`, `MultiSelectField`,
`CheckboxField`, `SwitchField`, `TextAreaField` (spread `form.field('name')` into them; 400 field errors
land on their fields), `FormError`, `ConfirmDialog` / `useConfirm()`, `useToast()`, `CopyButton`,
`CodeBlock`, `QueryTabs` (`?tab=`), `EmptyState`, `ErrorState`, `FormatBadge`, `TypeBadge`, `StatusBadge`,
`FileSize`, `DateTime`, `RelativeTime`, `UnsavedChangesGuard`. Pages must work at 390 px wide without
horizontal scrolling, in both themes.

## Tests

- Unit and component tests sit next to the code (`*.test.ts(x)`), run with Vitest and jsdom.
  `renderWithApp(element, { state, path, areas, routes })` from `src/test/render.tsx` gives the providers
  and a memory router; `fakeServer({...})` answers `fetch` per `METHOD path`.
- End-to-end: `e2e/*.spec.ts` with `test` from `e2e/fixtures.ts` (`server.url`, `server.adminPassword`,
  `server.setAnonymous()`, `signIn(page, password)`). The global setup starts `ASTRUM_BIN` (default
  `/home/user/wt/m1-target/debug/astrum`) on a free port (or `ASTRUM_E2E_PORT`) with a fresh data directory
  and a fresh database on `ASTRUM_E2E_DATABASE_URL` (default `postgres://postgres@127.0.0.1:5433/postgres`,
  name prefix `ASTRUM_E2E_DATABASE_PREFIX`), serving `dist/`; it drops both at the end. Chromium comes from
  `/opt/pw-browsers/chromium` (`ASTRUM_E2E_CHROMIUM`). Each page gets at least one scenario.
- Screenshots: `ASTRUM_SHOTS_DIR=<dir> corepack pnpm e2e screenshots` (1440x900 and 390x844, dark and
  light); add your pages to your own spec in the same way.
- The shared e2e server starts like an installation whose first-run setup is done (`admin.password` is
  removed after it is read). A scenario that needs a brand-new server calls
  `startServer({ firstRun: true, port })` from `e2e/server.ts` itself and stops it afterwards.
