
/*global Ext, NX*/

/**
 * Help button.
 *
 * @since 3.0
 */
Ext.define('NX.view.header.Help', {
  extend: 'Ext.button.Button',
  alias: 'widget.nx-header-help',
  requires: [
    'NX.I18n'
  ],

  iconCls: 'x-fa fa-question-circle', // fa-question-circle

  /**
   * @override
   */
  initComponent: function () {
    var me = this;

    me.tooltip = NX.I18n.get('Header_Help_Tooltip');
    me.ariaLabel =  NX.I18n.get('Header_Help_Tooltip');

    // hide the menu button arrow
    me.arrowCls = '';

    me.menu = [
      {
        text: NX.I18n.get('Header_Help_About_Text'),
        iconCls: 'nx-icon-new-astrum-black-x16',
        tooltip: NX.I18n.get('Header_Help_About_Tooltip'),
        action: 'about'
      },
      {
        text: NX.I18n.get('Header_Help_Documentation_Text'),
        iconCls: 'nx-icon-help-manual-x16',
        tooltip: NX.I18n.get('Header_Help_Documentation_Tooltip'),
        action: 'docs'
      },
      {
        text: NX.I18n.get('Header_Help_KB_Text'),
        iconCls: 'nx-icon-help-kb-x16',
        tooltip: NX.I18n.get('Header_Help_KB_Tooltip'),
        action: 'kb'
      },
      {
        text: NX.I18n.get('Header_Help_Guides_Text'),
        iconCls: 'nx-icon-help-guides-x16',
        tooltip: NX.I18n.get('Header_Help_Guides_Tooltip'),
        action: 'guides'
      },
      {
        text: NX.I18n.get('Header_Help_Community_Text'),
        iconCls: 'nx-icon-help-community-x16',
        tooltip: NX.I18n.get('Header_Help_Community_Tooltip'),
        action: 'community'
      },
      {
        text: NX.I18n.get('Header_Help_Issues_Text'),
        iconCls: 'nx-icon-help-issues-x16',
        tooltip: NX.I18n.get('Header_Help_Issues_Tooltip'),
        action: 'issues'
      },
      '-',
      {
        text: NX.I18n.get('Header_Help_Support_Text'),
        iconCls: 'nx-icon-help-support-x16',
        tooltip: NX.I18n.get('Header_Help_Support_Tooltip'),
        action: 'support'
      }
    ];

    me.callParent();
  }

});
