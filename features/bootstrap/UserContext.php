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
     * Creates the user with the given email address and password.
     *
     * @param string $email
     * @param string $password
     *
     * @Given user :email
     * @Given user :email with password :password
     */
    public function createUser(string $email, string $password = 'password'): void
    {
        $email = escapeshellarg($email);
        $password = escapeshellarg($password);
        $this->executeCommand("user add $email $password");
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
     */
    public function logInAs(string $email, string $password = 'password'): void
    {
        $this->users[$email] = $email;
        $this->createUser($email, $password);
        $this->activateUser($email);
        $this->visitPath('/user/login');
        $page = $this->getSession()->getPage();
        $page->fillField('Email address', $email);
        $page->fillField('Password', $password);
        $page->pressButton('Log in');
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
