
/*global Ext*/

/**
 * Plugin that shows a tooltip below a search box with some hints about syntax
 *
 * @since 3.14
 */
Ext.define('AS.ext.plugin.SearchBoxTip', {
  extend: 'Ext.plugin.Abstract',
  alias: 'plugin.searchboxtip',

  requires: [
    'Ext.tip.Tip',
    'AS.I18n',
    'AS.controller.Help'
  ],

  init: function(hostComponent) {
    var me = this;

    me.setCmp(hostComponent);

    me.tip = Ext.create('Ext.tip.Tip', {
      alignTarget: hostComponent,
      cls: 'as-search-tip',
      defaultAlign: 'tl-bl',
      html: me.message + '<br>' +
          '<span class="footer"><a target="_blank" rel="noopener" href="' + AS.controller.Help.getDocsUrl() + '/Searching+for+Components">' +
          AS.I18n.get('SearchBoxTip_LearnMore') + ' <i class="fa fa-external-link" /></a></span>',
      listeners: {
        afterrender: function() {
          me.tip.el.set({'data-for-id': hostComponent.getInputId()});
        }
      }
    });

    hostComponent.on('focus', me.onFocus, me);
    hostComponent.on('blur', me.onBlur, me);
    hostComponent.on('beforedestroy', me.onBeforeDestroy, me);

    me.boundOnClick = me.onWindowClick.bind(me);
  },

  onFocus: function() {
    window.addEventListener('click', this.boundOnClick);

    this.tip.show();
  },

  onBlur: function() {
    /*
   * Install a delayed handler on blur to detect cases where focus is
   * lost to another element without a click. e.g. TAB press
   */
    this.blurTimeoutId = window.setTimeout(this.hide.bind(this), 200);
  },

  onBeforeDestroy: function() {
    this.hide();

    if (this.tip) {
      this.tip.destroy();
    }
  },

  onWindowClick: function(event) {
    if (!this.getCmp().el.contains(event.target) && !this.tip.el.contains(event.target)) {
      this.hide();
    }

    if (this.blurTimeoutId) {
      // A click happened before the blurTimeout was executed and the appropriate
      // action was taken above. There's no longer a need for the timeout.
      window.clearTimeout(this.blurTimeoutId);
      this.blurTimeoutId = null;
    }
  },

  hide: function() {
    if (this.tip) {
      this.tip.hide();
    }
    window.removeEventListener('click', this.boundOnClick);
  }

});
