package keeper

import (
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	stakingtypes "github.com/cosmos/cosmos-sdk/x/staking/types"

	"github.com/cosmos/gravity-bridge/module/x/gravity/types"
)

// HandleEthEvent handles a given event by attesting it
// TODO: it's not clear the utility of this from the code. Explain what it does,
// provice example and where this is executed on the step-by-step incoming logic.
func (k Keeper) HandleEthEvent(ctx sdk.Context, event types.EthereumEvent, orchestratorAddr sdk.AccAddress) error {
	validatorAddr := k.GetOrchestratorValidator(ctx, orchestratorAddr)
	if validatorAddr == nil {
		validatorAddr = sdk.ValAddress(orchestratorAddr)
	}

	// return an error if the validator isn't in the active set
	validator := k.stakingKeeper.Validator(ctx, validatorAddr)
	if validator == nil {
		return sdkerrors.Wrap(stakingtypes.ErrNoValidatorFound, validatorAddr.String())
	} else if !validator.IsBonded() {
		return sdkerrors.Wrapf(sdkerrors.ErrUnauthorized, "validator %s not in active set", validatorAddr)
	}

	// Add the event to the store
	if err := k.AttestEvent(ctx, event, validatorAddr); err != nil {
		return sdkerrors.Wrap(err, "create attestation")
	}

	// Emit the handle message event
	ctx.EventManager().EmitEvent(
		sdk.NewEvent(
			sdk.EventTypeMessage,
			sdk.NewAttribute(sdk.AttributeKeyModule, event.GetType()),
			// TODO: maybe return something better here? is this the right string representation?
			// sdk.NewAttribute(types.AttributeKeyAttestationID, string(types.GetAttestationKey(event.GetEventNonce(), event.ClaimHash()))),
		),
	)

	return nil
}