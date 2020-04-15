<?php

declare(strict_types = 1);

namespace Firetrack\Tests\Traits;

use Firetrack\Tests\EmailMessage;

/**
 * Reusable code for interacting with activation codes.
 */
trait ActivationCodeTrait
{

    /**
     * Returns the first activation message that was sent to the given email address.
     *
     * @param string $email
     *   The email address to which the message was sent.
     *
     * @return EmailMessage|null
     *   The message, or NULL if no message was found.
     */
    protected function getActivationMessage(string $email): ?EmailMessage
    {
        assert(
            in_array(MailgunTrait::class, class_uses($this)),
            __METHOD__ . ' depends on MailgunTrait. Please include it in ' . __CLASS__
        );

        foreach ($this->getMessagesTo($email) as $message) {
            if ($this->isActivationMessage($message)) {
                return $message;
            }
        }

        return null;
    }

    /**
     * Returns the activation code contained in the given activation mail.
     *
     * @param EmailMessage $message
     *   The activation message.
     *
     * @return string
     *   The activation code.
     *
     * @throws \InvalidArgumentException
     *   Thrown when the passed in message is not an activation message.
     */
    protected function getActivationCodeFromMessage(EmailMessage $message): string
    {
        if (!$this->isActivationMessage($message)) {
            throw new \InvalidArgumentException('Can only retrieve activation codes from activation messages');
        }

        $matches = [];
        preg_match('/^Activation code: (\d{6})$/', $message->text, $matches);

        return $matches[1];
    }

    /**
     * Checks whether the passed in message is an activation message.
     *
     * @param EmailMessage $message
     *
     * @return bool
     */
    protected function isActivationMessage(EmailMessage $message): bool
    {
        return (bool)preg_match('/^Activation code: \d{6}$/', $message->text);
    }

    /**
     * Returns the activation code for the given email.
     *
     * @param string $email
     *
     * @return string
     *   The activation code.
     */
    protected function getActivationCode(string $email): string
    {
        assert(
            in_array(FiretrackCliTrait::class, class_uses($this)),
            __METHOD__ . ' depends on FiretrackCliTrait. Please include it in ' . __CLASS__
        );

        $result = $this->executeCommand('activation-code get ' . escapeshellarg($email));
        return reset($result);
    }

}
