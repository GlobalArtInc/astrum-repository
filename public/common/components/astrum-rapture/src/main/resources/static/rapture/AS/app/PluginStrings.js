
/*global Ext, AS*/

/**
 * Application strings
 *
 * @since 3.0
 */
Ext.define('AS.app.PluginStrings', {
  '@aggregate_priority': 90,

  singleton: true,

  requires: [
    'AS.I18n'
  ],

  //
  // Note: Symbols follow the following naming convention:
  // <Class>_<Name>_<Component or Attribute>
  //

  keys: {
    // Buttons
    Button_Back: 'Back',
    Button_Cancel: 'Cancel',
    Button_Close: 'Close',
    Button_Next: 'Next',

    Column_No_Data: 'No data',

    // Header
    Header_Panel_Logo_Text: 'Astrum Repository',
    User_Tooltip: 'Hi, {0}. Manage your user account.',
    Header_SignIn_Text: 'Sign in',
    Header_SignIn_Tooltip: 'Sign in',
    Header_SignOut_Text: 'Sign out',
    Header_SignOut_Tooltip: 'Sign out',
    Header_Help_Tooltip: 'Help',
    Header_Help_About_Text: 'About',
    Header_Help_About_Tooltip: 'About GlobalArt Astrum Repository',
    Header_Help_Documentation_Text: 'Documentation',
    Header_Help_Documentation_Tooltip: 'Product documentation',
    Header_Help_KB_Text: 'Knowledge base',
    Header_Help_KB_Tooltip: 'Knowledge base',
    Header_Help_Guides_Text: 'GlobalArt guides',
    Header_Help_Guides_Tooltip: 'GlobalArt guides',
    Header_Help_Community_Text: 'Community',
    Header_Help_Community_Tooltip: 'Community information',
    Header_Help_Issues_Text: 'Issue tracker',
    Header_Help_Issues_Tooltip: 'Issue and bug tracker',
    Header_Help_Support_Text: 'Support',
    Header_Help_Support_Tooltip: 'Product support',

    // Footer

    // Sign in
    SignIn_Username_Empty: 'Username',
    SignIn_Password_Empty: 'Password',

    // Filter box
    Grid_Plugin_FilterBox_Empty: 'Filter',

    // Dialogs
    Dialogs_Info_Title: 'Information',
    Dialogs_Error_Title: 'Error',
    Dialogs_Error_Message: 'Operation failed',

    // Server
    User_ConnectFailure_Message: 'Operation failed as server could not be contacted',
    State_Reconnected_Message: 'Server reconnected',
    State_Disconnected_Message: 'Server disconnected',
    UiSessionTimeout_Expire_Message: 'Session is about to expire',
    UiSessionTimeout_Expired_Message: 'Session expired after being inactive for {0} minutes',
    User_SignedIn_Message: 'User signed in: {0}',
    User_SignedOut_Message: 'User signed out',
    Util_DownloadHelper_Download_Message: 'Download initiated',
    Windows_Popup_Message: 'Window pop-up was blocked!',

    // License
    State_Installed_Message: 'License installed',
    State_Uninstalled_Message: 'License uninstalled',

    // About modal
    AboutWindow_Title: 'About GlobalArt Astrum Repository',
    AboutWindow_About_Title: 'Legal Terms and Copyright',

    // Authentication modal
    Authenticate_Title: 'Authenticate',
    Authenticate_Help_Text: 'You have requested an operation which requires validation of your credentials.',
    User_View_Authenticate_Submit_Button: 'Authenticate',
    User_Retrieving_Mask: 'Retrieving authentication token&hellip;',
    Authenticate_Cancel_Button: '@Button_Cancel',

    // Expiry modal
    ExpireSession_Title: 'Session',
    ExpireSession_Help_Text: 'Session is about to expire',
    UiSessionTimeout_Expire_Text: 'Session will expire in {0} seconds',
    SignedOut_Text: 'Your session has expired. Please sign in.',
    ExpireSession_Cancel_Button: '@Button_Cancel',
    ExpireSession_SignIn_Button: 'Sign in',

    // Unsaved changes modal

    // Unsupported browser

    // 404

    // Buttons

    // Item selector
    Form_Field_ItemSelector_Empty: 'Filter',

    // Settings form

    // Browse -> Welcome

    // Field validation messages
    Util_Validator_Text: 'Only letters, digits, underscores(_), hyphens(-), and dots(.) are allowed and may not start with underscore or dot.',
    Util_Validator_Hostname: 'Hostname must be valid',
    Util_Validator_Trim: 'This field may not start or end with a space',
    Util_Validator_Url: 'This field should be a URL in the format "http://www.example.com"',
    Util_Validator_Subdomain_Text: 'Subdomain field must be a minimum of 1 and maximum of 63 characters (letters, numbers, and dashes) and must start with a letter and end with a letter or digit',

    // Wizard
    Wizard_Next: '@Button_Next',
    Wizard_Back: '@Button_Back',
    Wizard_Cancel: '@Button_Cancel',
    Wizard_Screen_Progress: '{0} of {1}',

    // SearchBoxTip
    SearchBoxTip_LearnMore: 'Learn more...',

    // DependencySnippet Panel

    // Copy link column
    CopyLink_Button: 'copy',

    // Shared UI
    State_Disconnected_Title: 'Server disconnected',
    State_Disconnected_Text: 'There is a problem communicating with the server',
    State_Restarted_Title: 'Server restarted',
    State_Restarted_Text: 'Application will be reloaded as server has been restarted',
    ItemOrderer_Top_Tooltip: 'Move to Top',
    ItemOrderer_Up_Tooltip: 'Move Up',
    ItemOrderer_Down_Tooltip: 'Move Down',
    ItemOrderer_Bottom_Tooltip: 'Move to Bottom',
    ValueSet_Blank_Text: 'At least one value is required',
    ValueSet_Value_Header: 'Value',
    ValueSet_Delete_Tooltip: 'Delete',
    Grid_ClearSort_Text: 'Clear Sort',
    AboutWindow_Version_Label: 'Version',
    AboutWindow_Edition_Label: 'Edition',
    AboutWindow_BuildRevision_Label: 'Build Revision',
    AboutWindow_BuildTimestamp_Label: 'Build Timestamp',
    Util_Validator_OciUrl: 'Must be a valid URL using http://, https://, oci://, or oci+http://'
  }
}, function(obj) {
  AS.I18n.register(obj);
});

