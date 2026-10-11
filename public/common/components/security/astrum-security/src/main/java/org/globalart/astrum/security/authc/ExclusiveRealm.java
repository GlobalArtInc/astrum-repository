package org.globalart.astrum.security.authc;

import org.apache.shiro.realm.Realm;

/**
 * A realm that alone decides on the tokens it supports.
 *
 * When an active exclusive realm supports a token, {@link FirstSuccessfulModularRealmAuthenticator} asks no other
 * realm about it, so credentials that only make sense to this realm (such as an access token sent as a password) are
 * never offered to a directory, where failed attempts may lock the account.
 */
public interface ExclusiveRealm
    extends Realm
{
}
