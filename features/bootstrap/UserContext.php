<?php

declare(strict_types = 1);

use Behat\MinkExtension\Context\RawMinkContext;
use Firetrack\Tests\Exception\ExpectationException;

/**
 * Step definitions for interacting with the user profile pages.
 */
class UserContext extends RawMinkContext
{

    /**
     * A list of possible locations of the firetrack binary, relative to the current path, in order of preference.
     */
    const BINARY_LOCATIONS = [
        'target/release/cli',
        'target/debug/cli',
    ];

    /**
     * Navigates to the user registration form.
     *
     * @Given I am on the user registration form
     */
    public function goToUserRegistrationForm(): void
    {
        $this->visitPath('/user/register');
        $this->assertSession()->statusCodeEquals(200);
    }

    /**
     * Deletes the user with the given email address.
     *
     * @param string $email
     *
     * @Then I delete the user :email
     */
    public function deleteUser(string $email): void
    {
        $this->executeCommand('user delete ' . escapeshellarg($email));
    }

    /**
     * Executes a command using the Firetrack binary.
     *
     * @param string $command
     *   The command to execute.
     *
     * @return string[]
     *   The command output.
     */
    protected function executeCommand(string $command): array {
        $output = [];
        $exit = 0;

        $binary = $this->locateExecutable();
        $command = escapeshellcmd("$binary $command");
        exec($command, $output, $exit);

        if ($exit !== 0) {
            throw new ExpectationException(sprintf("Command '%s' returned error code %u.", $command, $exit));
        }

        return $output;
    }

    /**
     * Locates the Firetrack executable.
     *
     * @return string
     *   The absolute path to the Firetrack binary.
     */
    protected function locateExecutable(): string {
        foreach (static::BINARY_LOCATIONS as $location) {
            $path = getcwd() . DIRECTORY_SEPARATOR . $location;
            if (is_executable($path)) {
                return realpath($path);
            }
        }

        throw new ExpectationException(sprintf('Firetrack executable could not be located.'));
    }

}
