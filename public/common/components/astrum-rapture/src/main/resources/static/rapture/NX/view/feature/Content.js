
/*global Ext*/

/**
 * Content panel.
 *
 * @since 3.0
 */
Ext.define('NX.view.feature.Content', {
  extend: 'Ext.panel.Panel',
  requires: ['NX.view.feature.BreadcrumbPanel', 'NX.view.MaliciousRiskOnDisk', 'NX.State', 'NX.constants.FeatureFlags'],
  alias: 'widget.nx-feature-content',
  ariaRole: 'main',
  itemId: 'feature-content',
  ui: 'nx-feature-content',
  cls: 'nx-feature-content',
  layout: 'fit',

  /**
   * @private
   * If false, show a warning modal when you’re about to discard unsaved changes by navigating away
   */
  discardUnsavedChanges: false,

  dockedItems: [
    {
      xtype: 'nx-breadcrumb',
      dock: 'top'
    }
  ],

  listeners: {
    afterrender: function(obj) {
      obj.rendered = true;
      obj.showRoot();
    }
  },

  /**
   * Show the feature root (hide the breadcrumb)
   */
  showRoot: function() {
    var me = this;
    var breadcrumb = me.down('#breadcrumb');

    if (!me.rendered) {
      return;
    }

    if (breadcrumb.items.length !== 3) {
      breadcrumb.removeAll();
      breadcrumb.add(
          {
            xtype: 'container',
            itemId: 'nx-feature-icon',
            width: 32,
            height: 32,
            userCls: me.currentIcon,
            ariaRole: 'presentation'
          },
          {
            xtype: 'label',
            cls: 'nx-feature-name',
            text: me.currentTitle
          },
          {
            xtype: 'label',
            cls: 'nx-feature-description',
            text: me.currentDescription
          }
      );
    }
    else {
      breadcrumb.items.getAt(0).setUserCls(me.currentIcon);
      breadcrumb.items.getAt(1).setText(me.currentTitle);
      breadcrumb.items.getAt(2).setText(me.currentDescription);

      if (breadcrumb.items.length > 3) {
        Ext.each(breadcrumb.items.getRange(3), function(item) {
          breadcrumb.remove(item);
        });
      }
    }
  },

  /**
   * The currently set title and icon, so subpanels can access it
   */
  currentIcon: undefined,
  currentTitle: undefined,

  /**
   * Custom handling for icon
   *
   * @override
   * @param iconCls
   */
  setIconCls: function(iconCls) {
    this.currentIcon = iconCls;
  },

  /**
   * Custom handling for title
   *
   * @override
   * @param text
   */
  setTitle: function(text) {
    this.currentTitle = text;
  },

  /**
   * Set description text.
   *
   * @public
   * @param text
   */
  setDescription: function(text) {
    this.currentDescription = text;
  },

  /**
   * The currently set iconCls, so we can remove it when changed.
   *
   * @private
   */
  currentIconCls: undefined,

  /**
   * @public
   * Reset the discardUnsavedChanges flag (false by default)
   */
  resetUnsavedChangesFlag: function(enable) {
    var me = this;

    if (enable) {
      me.discardUnsavedChanges = true;
    }
    else {
      me.discardUnsavedChanges = false;
    }
  }
});
