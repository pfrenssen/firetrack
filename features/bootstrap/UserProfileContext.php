<?php

declare(strict_types = 1);

use Behat\MinkExtension\Context\RawMinkContext;

/**
 * Step definitions for interacting with the user profile pages.
 */
class UserProfileContext extends RawMinkContext
{

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

}
