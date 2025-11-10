
package org.globalart.astrum.crypto.secrets;

/**
 * Interface for encrypting and decrypting data.
 * 
 * @param <T> - data to be encrypted
 * @param <U> - encrypted data
 * @param <V> - encoded data
 *
 *          Also provide flow for encrypting data -> encoding data -> decoding data -> decrypting data.
 */
public interface EncryptDecrypt<T, U, V>
{
  default V encryptAndEncode(T toEncodeAdnEncrypt) {
    return encode(encrypt(toEncodeAdnEncrypt));
  }

  default T decodeAndDecrypt(V toDecodeAndDecrypt) {
    return decrypt(decode(toDecodeAndDecrypt));
  }

  V encode(U toEncode);

  U decode(V toDecode);

  U encrypt(T toEncrypt);

  T decrypt(U toDecrypt);

}
