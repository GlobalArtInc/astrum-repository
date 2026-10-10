
/*global Ext, AS*/

/**
 * Help button.
 *
 * @since 3.0
 */
Ext.define('AS.view.header.Help', {
  extend: 'Ext.button.Button',
  alias: 'widget.as-header-help',
  requires: [
    'AS.I18n'
  ],

  iconCls: 'x-fa fa-question-circle', // fa-question-circle

  /**
   * @override
   */
  initComponent: function () {
    var me = this;

    me.tooltip = AS.I18n.get('Header_Help_Tooltip');
    me.ariaLabel =  AS.I18n.get('Header_Help_Tooltip');

    // hide the menu button arrow
    me.arrowCls = '';

    me.menu = [
      {
        text: AS.I18n.get('Header_Help_About_Text'),
        iconCls: 'as-icon-new-astrum-black-x16',
        tooltip: AS.I18n.get('Header_Help_About_Tooltip'),
        action: 'about'
      },
      {
        text: AS.I18n.get('Header_Help_Documentation_Text'),
        iconCls: 'as-icon-help-manual-x16',
        tooltip: AS.I18n.get('Header_Help_Documentation_Tooltip'),
        action: 'docs'
      },
      {
        text: AS.I18n.get('Header_Help_KB_Text'),
        iconCls: 'as-icon-help-kb-x16',
        tooltip: AS.I18n.get('Header_Help_KB_Tooltip'),
        action: 'kb'
      },
      {
        text: AS.I18n.get('Header_Help_Guides_Text'),
        iconCls: 'as-icon-help-guides-x16',
        tooltip: AS.I18n.get('Header_Help_Guides_Tooltip'),
        action: 'guides'
      },
      {
        text: AS.I18n.get('Header_Help_Community_Text'),
        iconCls: 'as-icon-help-community-x16',
        tooltip: AS.I18n.get('Header_Help_Community_Tooltip'),
        action: 'community'
      },
      {
        text: AS.I18n.get('Header_Help_Issues_Text'),
        iconCls: 'as-icon-help-issues-x16',
        tooltip: AS.I18n.get('Header_Help_Issues_Tooltip'),
        action: 'issues'
      },
      '-',
      {
        text: AS.I18n.get('Header_Help_Support_Text'),
        iconCls: 'as-icon-help-support-x16',
        tooltip: AS.I18n.get('Header_Help_Support_Tooltip'),
        action: 'support'
      }
    ];

    me.callParent();
  }

});
