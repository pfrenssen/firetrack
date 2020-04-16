<?php

declare(strict_types = 1);

namespace Firetrack\Tests\Traits;

use Firetrack\Tests\EmailMessage;

/**
 * Reusable code for interacting with messages sent through Mailgun.
 */
trait MailgunTrait
{

    /**
     * Returns the messages which were sent to the given email address.
     *
     * @param string $email
     *   The email address to which the messages were sent.
     *
     * @return EmailMessage[]
     */
    protected function getMessagesTo(string $email): array
    {
        return array_filter(
            $this->getMessages(),
            function (EmailMessage $message) use ($email) {
                return $message->to === $email;
            }
        );
    }

    /**
     * Returns the list of email messages that have been sent via Mailgun.
     *
     * @return EmailMessage[]
     */
    protected function getMessages(): array
    {
        return array_map(
            function (string $line) {
                return EmailMessage::fromServerLogEntry($line);
            },
            $this->getLog()
        );
    }

    /**
     * Returns the Mailgun mock server log as an array of lines.
     *
     * @return string[]
     */
    protected function getLog(): array
    {
        // @todo Make the path to the log file configurable in behat.yml.
        $filename = getcwd() . '/mailgun-mock-server.log';
        return file($filename, FILE_IGNORE_NEW_LINES);
    }

}
