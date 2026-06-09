
/*global Ext, NX*/

/**
 * Help controller.
 *
 * @since 3.0
 */
Ext.define('NX.controller.Help', {
  extend: 'NX.app.Controller',
  requires: [
    'NX.Icons',
    'NX.I18n',
    'NX.Windows'
  ],

  views: [
    'header.Help',
    'AboutWindow'
  ],

  statics: {
    /**
     * The base-url for help links.
     *
     * @private
     * @property {String}
     * @readonly
     */
    baseUrl: 'https://links.globalart.dev/products/astrum',

    getDocsUrl: function() {
      return NX.controller.Help.baseUrl + '/docs/' + NX.State.getVersionMajorMinor();
    }
  },

  /**
   * @override
   */
  init: function () {
    var me = this;

    me.getApplication().getIconController().addIcons({
      'help-support': {
        file: 'support.png',
        variants: ['x16', 'x32']
      },
      'help-issues': {
        file: 'bug.png',
        variants: ['x16', 'x32']
      },
      'help-manual': {
        file: 'book_picture.png',
        variants: ['x16', 'x32']
      },
      'help-community': {
        file: 'users_4.png',
        variants: ['x16', 'x32']
      },
      'help-kb': {
        file: 'brain_trainer.png',
        variants: ['x16', 'x32']
      },
      'help-guides': {
        file: 'globalart.png',
        variants: ['x16', 'x32']
      }
    });

    me.listen({
      component: {
        'nx-header-help menuitem[action=about]': {
          click: me.onAbout
        },
        'nx-header-help menuitem[action=docs]': {
          click: me.onDocs
        },
        'nx-header-help menuitem[action=support]': {
          click: me.onSupport
        },
        'nx-header-help menuitem[action=issues]': {
          click: me.onIssues
        },
        'nx-header-help menuitem[action=community]': {
          click: me.onCommunity
        },
        'nx-header-help menuitem[action=kb]': {
          click: me.onKnowledgeBase
        },
        'nx-header-help menuitem[action=guides]': {
          click: me.onGuides
        }
      }
    });
  },

  /**
   * @private
   * @param {String} section
   */
  openUrl: function(section) {
    NX.Windows.open(NX.controller.Help.baseUrl + '/' + section);
  },

  /**
   * @private
   */
  onAbout: function() {
    Ext.widget('nx-aboutwindow');
  },

  /**
   * @private
   */
  onDocs: function() {
    NX.Windows.open(NX.controller.Help.getDocsUrl());
  },

  /**
   * @private
   */
  onSupport: function() {
    this.openUrl('support');
  },

  /**
   * @private
   */
  onIssues: function() {
    this.openUrl('issues');
  },

  /**
   * @private
   */
  onCommunity: function() {
    this.openUrl('community');
  },

  /**
   * @private
   */
  onKnowledgeBase: function() {
    this.openUrl('kb');
  },

  /**
   * @private
   */
  onGuides: function() {
    NX.Windows.open("https://links.globalart.dev/products/astrum/guides")
  }
});
