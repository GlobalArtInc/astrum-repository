# Web interface

The web interface is served at the address of the server, for example `https://astrum.example.com/`.
Users browse and search repositories, upload components, copy the settings their build tools need and
manage their access tokens there; administrators configure the server under **Settings**. Every page and
menu entry appears only for users who hold the privileges it needs, so an anonymous visitor may see no
more than the dashboard and the repositories open to everyone.

## Sign in

**Sign in** in the header, or any page that needs a user, opens the sign-in page: enter the **User name**
and **Password** and choose **Sign in**. Local and LDAP users sign in here with their password.

- **Continue with SSO** appears when [OpenID Connect](../configure/authentication.md#openid-connect) is
  configured and signs in through the provider.
- **Continue without signing in** appears while
  [anonymous access](../configure/authentication.md#anonymous-access) is on.
- After ten failed sign-ins of one user name within five minutes, the page refuses that name until the
  five minutes are over.

**Sign Out** is in the account menu, the round button with your initials at the right of the header. A
session stays open until you sign out, unless an administrator sets a **Session timeout** in the
capability **UI: Settings** (see [capabilities](../configure/capabilities.md)); changing your password ends
your other sessions.

### First start

On a new server the sign-in page shows where the generated password of `admin` is (`admin.password` in
the data directory). After signing in with it, the page **Set up Astrum Repository** walks through three
steps: **Password** sets a password of your own, **Anonymous access** chooses between **Allow anonymous
access** and **Require sign-in**, and **Repositories** creates hosted repositories with common settings
for the formats you pick. **Skip for now** leaves it; it comes back at the next sign-in until the
password of `admin` is changed.

## Header and menu

The main menu on the left holds **Dashboard**, **Search**, **Browse**, **Upload** and **API**, and below
them **Settings**. The button at the left of the header collapses the menu to icons; on a phone it opens
the menu as a drawer.

The header holds, from left to right:

- the search field, which also opens with `Ctrl K` (`⌘ K` on a Mac), see [search](#search);
- **Refresh**, which reloads the data of the page without reloading the page;
- the theme switch and the **Language** menu, see [theme and language](#theme-and-language);
- **System status**, a bell for users with the privilege `astrum:metrics:read`; a red dot on it means a
  health check fails, and it opens **Metrics and Health**;
- **GitHub**, a link to the project on GitHub with the number of its stars. The server asks
  `api.github.com` for the number, through the [HTTP settings](../configure/outbound-http.md), at most
  once in six hours (once in thirty minutes while the asking fails), so the page costs GitHub nothing
  per visitor and your browser never talks to GitHub. Where the server cannot reach GitHub, the link
  shows without a number;
- **Help and documentation**, with links to this documentation, the release notes and the issue tracker,
  and the version of the server;
- the account menu: **My Account**, **Access Tokens**, **NuGet API Key** (while the NuGet API-Key Realm is
  active) and **Sign Out**.

On a phone the **Language** and help menus move into the menu drawer, and the GitHub link is left out.

## Dashboard

**Dashboard** shows how the repositories you may browse were used over the last 7, 30 or 90 days, for all
of them or for the one chosen in **Repository**:

- tiles for **Downloads**, **Components**, **Files**, **Storage** (with the share of proxy caches),
  **Repositories** (hosted, proxy and group) and the space taken by components nobody downloaded for 90
  days;
- **Downloads per day**, with weekends in a darker colour;
- **Most downloaded**, for the period or of all time;
- **Repositories**, ordered **By downloads** or **By size**;
- **Recently published**;
- the components not downloaded for 90 days, with the repositories that hold the most of them and a link
  to **Configure cleanup policies**.

## Browse

**Browse** lists the repositories you may open, with their format, type, status (**Online** or
**Offline**) and address. Filter them by name, or by format, type and status under **Filters**. The
actions of a row are **Open Repository**, **Copy URL** and, for users who may change it, **Repository
Settings**.

A repository opens with its content as a tree on the left and the details of the selected item on the
right; drag the border between them to change the width of the tree. The filter above the tree narrows
the folders already loaded, and the search field of the page finds components in the repository, or in
the members of a group, with a link to the full search.

Selecting an item in the tree shows:

- for a folder: its path, depth and the number of folders and files in it. **Delete Folder** deletes
  everything in it that you may delete and keeps the rest;
- for a component: its group, name, version and downloads, the [usage snippets](#usage-snippets) and
  **Delete Component**;
- for a file: **Download**, **Delete File**, the usage snippets, a preview, and its **Details**: content
  type, size, who uploaded it, when it was created and last downloaded, its downloads and its blob
  reference, followed by the attributes of its format.

### File preview

The **Preview** of a file shows

- images (PNG, JPEG, GIF, WebP, AVIF, BMP, icons and SVG) up to 8 MiB;
- text in UTF-8 up to 1 MiB, with line numbers and a button that copies the whole file;
- Markdown files rendered, or as text with **Source**.

Archives, packages and other binary files have no preview, and larger files only say that they are too
large; download them instead.

### Component pages

The components that the search field of the header suggests, and **Versions in all repositories** in the
search results, open a page of the component with the tabs **Overview**, **Versions**, **Repositories** and
**Files**. **Overview** holds **Dependencies**, a snippet that adds this version of
the component to a build (for example the `<dependency>` of a `pom.xml`, chosen from the list next to the
copy button), and the **Component Details**. **Versions** lists every version, **Repositories** the
repositories that hold them, and **Files** the files; the tables can be filtered and exported to CSV.

## Usage snippets

Components and files of most formats show **Usage**: one tab per client, for example Maven and Gradle
for a Maven repository, each with a short explanation and a snippet to copy. The snippets name the
repository by its address, `/repository/<name>/` after the address of the server (the
[base URL](../configure/runtime.md#base-url) when one is set), or by the domain of its own when the
repository has one. They cover reading from the repository and, where the format and the repository
allow it, publishing to it. The settings of every client are described in
[connect build tools](clients.md) and on the page of each format.

## Search

The search field of the header lists your recent searches and, as you type, the repositories whose
name matches and the first matching components. **Enter** opens the search page for the text in all
repositories; while a repository is open, the list also offers to search only in it.

**Search** finds components in every repository you may see. Type at least two characters, or pick a
**Format** or a **Repository**:

- the text is looked for in the name, group and version of components, some descriptive attributes of
  their format, and the paths of their files, ignoring case;
- `*` stands for any characters;
- with several words, a component matches when any of them does.

The results show the component, version, format, repository and the time of the last change, sorted as
chosen under **Sort**; **Show More** loads the next results. A result opens the component in its
repository, and **Versions in all repositories** opens its [component page](#component-pages).

**Advanced Search**, at `/search/custom`, combines several conditions that must all match. Each condition
is a field (**Keyword**, **Format**, **Repository**, **Group**, **Name**, **Version**, a checksum, or an
attribute of a format such as **Maven Group ID** or **Docker Image Tag**), a condition (**Equals**,
**Starts with**, **Contains**) and a value. The same searches are available over the
[REST API](rest-api.md#search-components-and-assets).

## Upload

**Upload** lists the hosted repositories you may add to whose format takes uploads through the web
interface. Choose one, or choose the **Target Repository** on the form, then fill in the fields of the
format, such as **Group ID**, **Artifact ID** and **Version** for Maven or **Directory** for raw files, and
drop the files onto the form or click to choose them. **Add Another File** adds files to the same
component, each with its own fields, for example a **Classifier** and an **Extension**. **Upload
Component** sends the form and shows the progress; **Cancel** stops it. Scripts upload the same way
through the [REST API](rest-api.md#upload-a-component).

## Your account

**Account menu → My Account** shows your user name and where the account comes from (**Local**, **LDAP**
or **OpenID Connect**). Local users can change their first name, last name and email there and change
their password, which needs at least 8 characters; accounts from a directory or an identity provider are
managed there and are read-only.

**Account menu → Access Tokens** lists your [access tokens](../configure/authentication.md#access-tokens)
and when each was last used:

- **Create Token** asks for a name, the **Access** (**Read only** or **Read and write**) and a
  **Lifetime** of 7, 30 or 90 days, 1 year or never. The token is shown once; copy it before closing the
  dialog.
- **Revoke** cuts a token off at once.
- Users who may see the tokens of others switch between **My Tokens** and **All Users**.
- **Connected Agents** lists the agents connected through [OAuth for MCP](mcp.md), each with
  **Disconnect**.

Tokens can only be created and revoked after signing in with a password, not with a token.

**Account menu → NuGet API Key** shows or resets the API key that NuGet clients push with.

## Settings

**Settings** opens a page of tiles in four groups, with a search field above them. Only the tiles you have
privileges for are shown, and **Settings** disappears from the menu when there are none.

**Repository**

- **Repositories**: create, change and delete hosted, proxy and group repositories. A repository has
  the tabs **Summary**, **Settings**, **Audit**, **Tasks & Capabilities** and **Access & Security**, and
  the actions **Take Offline**, **Invalidate Cache**, **Rebuild Index** and **Delete**. See
  [repositories](../repositories/overview.md).
- **Blob Stores**: where the files of repositories are kept, on disk, in S3 or on other servers. See
  [blob stores](../configure/blob-stores.md).
- **Cleanup Policies**: rules that delete components nobody needs, with a preview of what they would
  delete. See [cleanup policies](../repositories/cleanup-policies.md).
- **Routing Rules**: allow or block the requests of proxy repositories by path. See
  [routing rules](../repositories/routing-rules.md).
- **Content Selectors**: expressions that pick out content for privileges. See
  [users, roles and privileges](../configure/authentication.md#users-roles-and-privileges).

**Security**

- **Users**, **Roles** and **Privileges**: who may do what. See
  [users, roles and privileges](../configure/authentication.md#users-roles-and-privileges).
- **Realms**: which ways of signing in are active, and in what order. See
  [realms](../configure/authentication.md#realms).
- **LDAP**: sign-in with directory accounts. See [LDAP](../configure/authentication.md#ldap).
- **Default Role**: a role every signed-in user gets.
- **OpenID Connect**: sign-in through an identity provider. See
  [OpenID Connect](../configure/authentication.md#openid-connect).
- **Anonymous Access**: what callers without credentials may do. See
  [anonymous access](../configure/authentication.md#anonymous-access).
- **SSL Certificates**: certificates the server trusts for its outgoing connections. See
  [SSL certificates](../configure/ssl-certificates.md).

**Support**

- **System Information**: version, environment and diagnostics of the server.
- **Logs** and **Logging**: read the log file and change log levels at runtime. See
  [logs](../operate/monitoring.md#logs).
- **Metrics and Health**: the health checks. See [monitoring](../operate/monitoring.md#health-endpoints).
- **Support ZIP**: one archive with diagnostics for troubleshooting. See
  [support ZIP](../operate/monitoring.md#support-zip).

**System**

- **Tasks**: scheduled and manual maintenance, and how each run went. See [tasks](../operate/tasks.md).
- **Email Server**: the mail server that notifications and task alerts are sent through.
- **HTTP**: the proxy, timeouts and retries of outgoing requests. See
  [outbound HTTP](../configure/outbound-http.md).
- **API**: the REST API rendered from its OpenAPI document, where requests can be tried from the browser.
  See [REST API](rest-api.md#the-api-page).
- **Capabilities**: optional features such as the base URL, the Prometheus metrics and space alerts for
  blob stores. See [capabilities](../configure/capabilities.md).
- **Notifications**: channels in Telegram, Slack, Mattermost, Rocket.Chat, email or your own webhook that
  hear about published, updated and deleted components, and receive the
  [blob store space alerts](../operate/monitoring.md#blob-store-space-alerts).
- **Nodes**: the nodes of the installation and their state. See [nodes](../operate/nodes.md).

The **Audit Log**, at `/audit`, lists security and configuration changes and downloads, with filters by
period, category, format, repository, event type and initiator. It needs the privilege
`astrum:audit:read`, and the **Audit** tab of a repository links to it with **Full Audit Log**.

## Theme and language

The interface starts in the dark theme. The sun or moon button in the header switches between dark and
light, and the browser remembers the choice.

The **Language** menu in the header, also on the sign-in page, switches the interface between its twelve
languages at once; see [interface language](language.md).
