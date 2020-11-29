<?php

declare(strict_types = 1);

use Behat\MinkExtension\Context\RawMinkContext;
use Firetrack\Tests\Traits\ActivationCodeTrait;
use Firetrack\Tests\Traits\FiretrackCliTrait;

/**
 * Step definitions for interacting with the user profile pages.
 */
class UserContext extends RawMinkContext
{

    use ActivationCodeTrait;
    use FiretrackCliTrait;

    /**
     * Users created during scenarios.
     *
     * @var string[]
     *   An array of email addresses of users created during scenarios.
     */
    protected array $users = [];

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
     * Navigates to the user login form.
     *
     * @Given I am on the user login form
     */
    public function goToUserLoginForm(): void
    {
        $this->visitPath('/user/login');
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
     * Creates the user with the given email address, password and activation state.
     *
     * @param string $email
     * @param string $password
     *   The password, defaults to 'password'.
     * @param string $activation
     *   The activation state, can be either 'active' or 'inactive'. Defaults to 'active'.
     *
     * @Given user :email
     * @Given :activation user :email
     * @Given user :email with password :password
     * @Given :activation user :email with password :password
     */
    public function createUser(string $email, string $password = 'password', string $activation = 'active'): void
    {
        $escaped_email = escapeshellarg($email);
        $escaped_password = escapeshellarg($password);
        $this->executeCommand("user add $escaped_email $escaped_password");

        $this->users[$email] = $email;

        if ($activation === 'active') {
            $this->activateUser($email);
        }
    }

    /**
     * Activates the user with the given email address.
     *
     * @param string $email
     *
     * @Given the account of :email is activated
     */
    public function activateUser(string $email): void
    {
        $code = $this->getActivationCode($email);

        $email = escapeshellarg($email);
        $code = escapeshellarg($code);
        $this->executeCommand("user activate $email $code");

        // Create a set of default categories for the activated user.
        // Todo: This should be done automatically.
        // Ref. https://github.com/pfrenssen/firetrack/issues/193
        $this->executeCommand("category populate $email");
    }

    /**
     * Creates a user account for the given email address and logs in as this user.
     *
     * @param string $email
     * @param string $password
     *
     * @Given I am logged in as :email
     * @Given I am logged in as :email with password :password
     *
     * @throws \Behat\Mink\Exception\ElementNotFoundException
     *   Thrown when the form elements to log in a user are not found.
     * @throws \Behat\Mink\Exception\ResponseTextException
     *   Thrown when the success message is not shown after logging in.
     */
    public function logInAs(string $email, string $password = 'password'): void
    {
        $this->createUser($email, $password);
        $this->visitPath('/user/login');
        $page = $this->getSession()->getPage();
        $page->fillField('Email address', $email);
        $page->fillField('Password', $password);
        $page->pressButton('Log in');
        $this->assertSession()->pageTextContains('Log out');
    }

    /**
     * Cleans up users created during the scenario.
     *
     * @AfterScenario
     */
    public function cleanUsers(): void {
        foreach ($this->users as $email) {
            $this->deleteUser($email);
        }
        $this->users = [];
    }

}
