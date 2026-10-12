
/*global Ext, AS*/

/**
 * CoreUi plugin strings.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.app.PluginStrings', {
  '@aggregate_priority': 90,

  singleton: true,
  requires: [
    'AS.I18n'
  ],

  /**
   * String keys.
   *
   * Keys follow the following naming convention:
   *
   * Class_Name>_[Component_or_Attribute]: string
   *
   * @type {Object}
   */
  keys: {
    // Onboarding -> Change admin password
    User_UserChangePassword_NoMatch_Error: 'Passwords do not match',
    User_UserChangePassword_Password_FieldLabel: 'New password',
    User_UserChangePassword_PasswordConfirm_FieldLabel: 'Confirm password'
  }
}, function(self) {
  AS.I18n.register(self);
});
