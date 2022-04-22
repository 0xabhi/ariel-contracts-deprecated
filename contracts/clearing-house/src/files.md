## State

[/] curve_history

[/] deposit_history

[/] funding_payment_history
    merged into funding_history

[/] funding_rate_history
    merged into funding_history

[/] liquidation_history

[/] order_history

[/] trade_history

[/] market
    move amm impl to helpers/oracle

[/] order_state
    merged into order

[/] state

[/] user_orders
    merged into order

[/] user

## helpers/math

[/] amm
    moved update_mark_twap, update_oracle_price_twap to controller/amm

[/] bn

[/] casting

[/] collateral

[/] constants

[/] fees
    flipped try_<>_

[/] funding

[/] margin
    moved to controller

[/] oracle
    #TODO
        get Oracle price function

[/] orders
    calculate_base_asset_amount_user_can_execute, calculate_available_quote_asset_user_can_execute to controller/order
    recieved from /order_validation

[/] pnl

[/] positions

[/] quote_assets

[/] repeg
    moved to controller/repeg

[/] slippage

[/] withdrawal

## Controllers

[/] amm
    recieved update_mark_twap, update_oracle_price_twap from helpers/amm

[/] funding

[/] margin
    recieved from helpers/margin
    recieved from /margin_validation

[/] order
    recieved calculate_base_asset_amount_user_can_execute, calculate_available_quote_asset_user_can_execute from helpers/order

[/] position
    
[/] repeg
    recieved from helpers/repeg
