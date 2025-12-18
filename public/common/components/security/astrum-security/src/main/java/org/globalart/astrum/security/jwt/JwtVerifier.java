
package org.globalart.astrum.security.jwt;

import org.globalart.astrum.security.JwtHelper;

import com.auth0.jwt.JWT;
import com.auth0.jwt.JWTVerifier;
import com.auth0.jwt.algorithms.Algorithm;
import com.auth0.jwt.exceptions.JWTVerificationException;
import com.auth0.jwt.interfaces.DecodedJWT;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * Perform the verification against the given JWT,
 *
 * @since 3.38
 */
public class JwtVerifier
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final Algorithm algorithm;

  private final JWTVerifier verifier;

  private final String secret;

  public JwtVerifier(final String secret) {
    this.secret = secret;
    this.algorithm = Algorithm.HMAC256(secret);
    this.verifier = JWT.require(algorithm)
        .withIssuer(JwtHelper.ISSUER)
        .build();
  }

  /**
   * Perform the verification against the given Token.
   *
   * @param jwt the JWT to verify.
   * @return the {@link DecodedJWT} object.
   */
  public DecodedJWT verify(final String jwt) throws JwtVerificationException {
    DecodedJWT decodedJWT;
    try {
      decodedJWT = verifier.verify(jwt);
    }
    catch (JWTVerificationException e) {
      String errorMsg = "Can't verify the token";
      log.debug(errorMsg, e);
      throw new JwtVerificationException(errorMsg);
    }
    return decodedJWT;
  }

  public Algorithm getAlgorithm() {
    return algorithm;
  }

  public String getSecret() {
    return secret;
  }
}
