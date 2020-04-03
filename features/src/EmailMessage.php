<?php

declare(strict_types = 1);

namespace Firetrack\Tests;

/**
 * Represents an email message which has been sent through Mailgun.
 */
class EmailMessage
{

    public string $from;
    public string $to;
    public string $subject;
    public string $text;

    /**
     * Constructs a new EmailMessage.
     *
     * @param string $from
     * @param string $to
     * @param string $subject
     * @param string $text
     */
    public function __construct(string $from, string $to, string $subject, string $text)
    {
        $this->from = $from;
        $this->to = $to;
        $this->subject = $subject;
        $this->text = $text;
    }

    /**
     * Creates a new EmailMessage from a log entry from the Mailgun mock server.
     *
     * @param string $logEntry
     *   A URL encoded string representing the content of the email message.
     *
     * @return static
     */
    public static function fromServerLogEntry(string $logEntry): self
    {
        $parts = [];
        parse_str(urldecode($logEntry), $parts);
        return new static($parts['from'], $parts['to'], $parts['subject'], $parts['text']);
    }

}
